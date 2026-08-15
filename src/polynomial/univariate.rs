use crate::field::Fp;

pub struct UnivariatePolynomial {
    pub coefficients: Vec<Fp>,
}

impl UnivariatePolynomial {

    pub fn new(coefficients: Vec<Fp>) -> Self {
        assert!(
            !coefficients.is_empty(),
            "polynomial must have at least one coefficient"
        );
        UnivariatePolynomial { coefficients }
    }

    pub fn evaluate(&self, x: Fp) -> Fp {
        let mut output = Fp::zero();
        // using horners rule
        for &coeff in self.coefficients.iter().rev() {
            output = output * x + coeff;
        }
        output
    }

    pub fn degree(&self) -> usize {
        self.coefficients.len() - 1
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_at_zero_gives_constant_term() {
        let p = UnivariatePolynomial::new(vec![Fp::new(7), Fp::new(11), Fp::new(13)]);
        assert_eq!(p.evaluate(Fp::zero()), Fp::new(7));
    }

    #[test]
    fn evaluate_at_field_element_correct() {
        let p = UnivariatePolynomial::new(vec![Fp::new(3), Fp::new(5), Fp::new(2)]);
        // 3 + 5(2) + 2(2²) = 21
        assert_eq!(p.evaluate(Fp::new(2)), Fp::new(21));
    }

    #[test]
    fn degree_is_correct() {
        let p = UnivariatePolynomial::new(vec![Fp::new(7), Fp::new(11), Fp::new(13)]);
        assert_eq!(p.degree(), 2);
    }

    #[test]
    #[should_panic(expected = "at least one coefficient")]
    fn new_panics_on_empty_coefficients() {
        UnivariatePolynomial::new(vec![]);
    }
}