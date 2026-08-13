use crate::field::Fp;

#[derive(Clone, Copy, Debug)]
pub enum Gate {
    Add(usize, usize),
    Mul(usize, usize),
}
pub struct Circuit {
    pub layers: Vec<Vec<Gate>>,
    pub n: usize,
}

impl Circuit {
    
    pub fn example() -> Self {
        // the circuit computing (a + b) * (c + d):
        Circuit {
            layers: vec![
                vec![Gate::Mul(0, 1)], // layer 0
                vec![Gate::Add(0, 1), Gate::Add(2, 3)], // layer 1
            ], 
            n: 4
        }
    }

    pub fn evaluate(&self, inputs: &[Fp]) -> Vec<Vec<Fp>> {
        assert_eq!(inputs.len(), self.n, "wrong number of inputs");

        let layer_count = self.layers.len();
        let mut witness: Vec<Vec<Fp>> = vec![Vec::new(); layer_count + 1];
        witness[layer_count] = inputs.to_vec();

        for i in (0..layer_count).rev() {
            let prev_layer = &witness[i + 1];
            witness[i] = self.layers[i].iter()
                .map(|gate| match *gate {
                    Gate::Add(left, right) => prev_layer[left] + prev_layer[right],
                    Gate::Mul(left, right) => prev_layer[left] * prev_layer[right],
                })
                .collect();
        }
        witness
    }

    pub fn output(&self, inputs: &[Fp]) -> Fp {
        self.evaluate(inputs)[0][0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
 
    #[test]
    fn example_computes_sum_product() {
        let circuit = Circuit::example();
        let inputs = [Fp::new(2), Fp::new(3), Fp::new(4), Fp::new(5)];
        // (2 + 3) * (4 + 5) = 5 * 9 = 45
        assert_eq!(circuit.output(&inputs), Fp::new(45));
    }
 
    #[test]
    fn witness_has_one_entry_per_layer_plus_inputs() {
        let circuit = Circuit::example();
        let inputs = [Fp::new(1), Fp::new(1), Fp::new(1), Fp::new(1)];
        let witness = circuit.evaluate(&inputs);
 
        assert_eq!(witness.len(), 3);
        assert_eq!(witness[2], vec![Fp::new(1); 4]); // inputs (above)
        assert_eq!(witness[1], vec![Fp::new(2), Fp::new(2)]); // 1+1, 1+1
        assert_eq!(witness[0], vec![Fp::new(4)]); // 2*2
    }
 
    #[test]
    #[should_panic(expected = "wrong number of inputs")]
    fn evaluate_panics_on_wrong_input_length() {
        let circuit = Circuit::example();
        circuit.evaluate(&[Fp::new(1), Fp::new(2)]);
    }
}