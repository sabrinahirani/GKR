use crate::field::Fp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultilinearExtension {
    pub evaluations: Vec<Fp>,
}

impl MultilinearExtension {

    pub fn new(evaluations: Vec<Fp>) -> Self {
        // since circuit layer extension (using boolean hypercube):
        assert!(
            evaluations.len().is_power_of_two(),
            "number of evaluations must be a power of two"
        );
        MultilinearExtension { evaluations }
    }

    pub fn variable_count(&self) -> usize {
        // gives the number of variables n from evaluations.len() = 2^n
        self.evaluations.len().ilog2() as usize
    }

    pub fn fix_variable(&self, r: Fp) -> Self {
        assert!(
            self.evaluations.len() > 1,
            "cannot fix a variable in a 0-variable extension"
        );
        // we want to fix some variable x = r
        // we have:
        // x = 0 for the top of the evaluation table
        // x = 1 for the bottom of the evaluation table
        // so:
        let mid = self.evaluations.len() / 2;
        let mut new_evaluations = Vec::with_capacity(mid);
        for i in 0..mid {
            let top = self.evaluations[i]; // x = 0
            let bottom = self.evaluations[mid + i]; // x = 1
            // (1 - r) * top + r * bottom
            // note that if r = 0 then x = 0 and if r = 1 then x = 1 so multilinear extension
            new_evaluations.push(top + r * (bottom - top));
        }
        MultilinearExtension::new(new_evaluations)
    }

    pub fn evaluate(&self, point: &[Fp]) -> Fp {
        assert_eq!(
            point.len(),
            self.variable_count(),
            "point has the wrong number of coordinates"
        );
        let mut mle = self.clone();
        // evaluate in each coordinate
        for &r in point {
            mle = mle.fix_variable(r);
        }
        mle.evaluations[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "power of two")]
    fn new_panics_if_not_power_of_two() {
        MultilinearExtension::new(vec![Fp::new(1), Fp::new(2), Fp::new(3)]);
    }

    #[test]
    fn fix_variable_halves_the_table() {
        let evaluations = vec![Fp::new(10), Fp::new(20), Fp::new(30), Fp::new(40)];
        let mle = MultilinearExtension::new(evaluations);
        let folded = mle.fix_variable(Fp::new(2));
 
        assert_eq!(folded.evaluations.len(), 2);
        assert_eq!(folded.evaluations, vec![Fp::new(50), Fp::new(60)]); // (10 + 2*(30-10), 20 + 2*(40-20)) = (50, 60)
    }

    #[test]
    fn evaluate_matches_table_at_boolean_points() {
        let evaluations = vec![Fp::new(10), Fp::new(20), Fp::new(30), Fp::new(40)];
        let mle = MultilinearExtension::new(evaluations);
 
        let zero = Fp::zero();
        let one = Fp::one();
        assert_eq!(mle.evaluate(&[zero, zero]), Fp::new(10));
        assert_eq!(mle.evaluate(&[zero, one]), Fp::new(20));
        assert_eq!(mle.evaluate(&[one, zero]), Fp::new(30));
        assert_eq!(mle.evaluate(&[one, one]), Fp::new(40));
    }
 
    #[test]
    fn evaluate_matches_brute_force_at_non_boolean_point() {
        let evals = vec![Fp::new(10), Fp::new(20), Fp::new(30), Fp::new(40)];
        let mle = MultilinearExtension::new(evals.clone());
        let point = [Fp::new(3), Fp::new(5)];
 
        assert_eq!(mle.evaluate(&point), brute_force_evaluate(&evals, &point));
    }
 
    #[test]
    fn zero_variable_extension_is_a_constant() {
        let mle = MultilinearExtension::new(vec![Fp::new(7)]);
        assert_eq!(mle.evaluate(&[]), Fp::new(7));
    }
}