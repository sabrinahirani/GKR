use crate::field::Fp;
use crate::polynomial::{UnivariatePolynomial, MultilinearExtension};
use crate::transcript::Transcript;

pub struct SumcheckProof {
    pub round_polynomials: Vec<UnivariatePolynomial>,
}

pub fn prove(poly: &MultilinearExtension, transcript: &mut Transcript) -> (Fp, SumcheckProof) {

    let claimed_sum = poly.evaluations.iter().fold(Fp::zero(), |acc, &v| acc + v);
    transcript.append_scalar(b"claimed_sum", claimed_sum);

    let n_variables = poly.variable_count();
    let mut round_polynomials = Vec::with_capacity(n_variables);

    // round polynomial p_i(X) = sum_{x_{i+1},...,x_n in {0,1}} p(r_1, ..., r_{i-1}, X, x_{i+1}, ..., x_n)
    let mut p = poly.clone();

    // note that summing all evaluations gives the running sum
    // then partitioning over x_i gives the round polynomial for round i
    for _ in 0..n_variables {
        let mid = p.evaluations.len() / 2;
        let mut eval_at_0 = Fp::zero();
        let mut eval_at_1 = Fp::zero();
        for i in 0..mid {
            eval_at_0 = eval_at_0 + p.evaluations[i];
            eval_at_1 = eval_at_1 + p.evaluations[mid + i];
        }

        // multilinear so linear p_i(X) = b + aX agrees with the running sum when X \in {0, 1}
        let round_poly = UnivariatePolynomial::new(vec![eval_at_0, eval_at_1 - eval_at_0]);
        transcript.append_scalars(b"round_poly", &round_poly.coefficients);
        
        let r = transcript.challenge_scalar(b"challenge");
        p = p.fix_variable(r);

        round_polynomials.push(round_poly);
    }

    (claimed_sum, SumcheckProof { round_polynomials })
} 

#[derive(Debug, PartialEq, Eq)]
pub enum VerificationError {
    RoundCountMismatch,
    DegreeTooHigh { round: usize },
    InconsistentRoundSum { round: usize },
    FinalEvaluationMismatch,
}

pub fn verify(
    claimed_sum: Fp,
    n_variables: usize,
    proof: &SumcheckProof,
    transcript: &mut Transcript,
    oracle: impl Fn(&[Fp]) -> Fp
) -> Result<(), VerificationError> {

    if proof.round_polynomials.len() != n_variables {
        return Err(VerificationError::RoundCountMismatch);
    }

    transcript.append_scalar(b"claimed_sum", claimed_sum);

    let mut expected = claimed_sum;
    let mut challenges = Vec::with_capacity(n_variables);

    for (round, poly) in proof.round_polynomials.iter().enumerate() {
        
        // consistency check
        let round_sum = poly.evaluate(Fp::zero()) + poly.evaluate(Fp::one());
        if round_sum != expected {
            return Err(VerificationError::InconsistentRoundSum { round });
        }
        // degree check
        if poly.coefficients.len() > 2 {
            return Err(VerificationError::DegreeTooHigh { round });
        }

        transcript.append_scalars(b"round_poly", &poly.coefficients);
        
        let r = transcript.challenge_scalar(b"challenge");
        challenges.push(r);

        // update running sum
        expected = poly.evaluate(r);
    }

    // final check
    if oracle(&challenges) != expected {
        return Err(VerificationError::FinalEvaluationMismatch);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcript::Transcript;

    fn test_mle() -> MultilinearExtension {
        // evaluations over {0,1}^2 in order: (0,0),(0,1),(1,0),(1,1)
        MultilinearExtension::new(vec![
            Fp::new(1),
            Fp::new(2),
            Fp::new(3),
            Fp::new(4),
        ])
    }

    fn oracle_for(poly: &MultilinearExtension) -> impl Fn(&[Fp]) -> Fp + '_ {
        move |point: &[Fp]| poly.evaluate(point)
    }

    #[test]
    fn honest_proof_verifies() {
        let poly = test_mle();

        let mut prover_transcript = Transcript::new(b"sumcheck-test");
        let (claimed_sum, proof) = prove(&poly, &mut prover_transcript);

        let mut verifier_transcript = Transcript::new(b"sumcheck-test");
        let result = verify(claimed_sum,
            poly.variable_count(),
            &proof,
            &mut verifier_transcript,
            oracle_for(&poly),
        );

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn incorrect_sum_is_rejected() {
        let poly = test_mle();

        let mut prover_transcript = Transcript::new(b"sumcheck-test");
        let (claimed_sum, proof) = prove(&poly, &mut prover_transcript);

        let mut verifier_transcript = Transcript::new(b"sumcheck-test");
        let result = verify(
            claimed_sum + Fp::one(), // incorrect sum
            poly.variable_count(),
            &proof,
            &mut verifier_transcript,
            oracle_for(&poly),
        );

        assert_eq!(result, Err(VerificationError::InconsistentRoundSum { round: 0 }));
    }

        #[test]
    fn round_count_mismatch_is_rejected() {
        let poly = test_mle();

        let mut prover_transcript = Transcript::new(b"sumcheck-test");
        let (claimed_sum, mut proof) = prove(&poly, &mut prover_transcript);

        // drop a round polynomial
        proof.round_polynomials.pop(); 

        let mut verifier_transcript = Transcript::new(b"sumcheck-test");
        let result = verify(
            claimed_sum,
            poly.variable_count(),
            &proof,
            &mut verifier_transcript,
            oracle_for(&poly),
        );

        assert_eq!(result, Err(VerificationError::RoundCountMismatch));
    }

    #[test]
    fn tampered_round_polynomial_is_rejected() {
        let poly = test_mle();

        let mut prover_transcript = Transcript::new(b"sumcheck-test");
        let (claimed_sum, mut proof) = prove(&poly, &mut prover_transcript);

        // tamper with coefficient in round polynomial
        proof.round_polynomials[0].coefficients[0] = proof.round_polynomials[0].coefficients[0] + Fp::one();

        let mut verifier_transcript = Transcript::new(b"sumcheck-test");
        let result = verify(
            claimed_sum,
            poly.variable_count(),
            &proof,
            &mut verifier_transcript,
            oracle_for(&poly),
        );

        assert_eq!(result, Err(VerificationError::InconsistentRoundSum { round: 0 }));
    }

    #[test]
    fn over_degree_round_polynomial_is_rejected() {
        let poly = test_mle();

        let mut prover_transcript = Transcript::new(b"sumcheck-test");
        let (claimed_sum, mut proof) = prove(&poly, &mut prover_transcript);

        // note X * (X-1) = 0 vanishes at both X=0 and X=1 so:
        // adding c * X * (X - 1) = c*X^2 - c*X to the polynomial changes the degree
        // without changing the value at the two boundary points X=0 and X=1
        let c = Fp::new(7);
        proof.round_polynomials[0].coefficients[1] = proof.round_polynomials[0].coefficients[1] - c; // adjust the X coefficient
        proof.round_polynomials[0].coefficients.push(c);     // add the X^2 coefficient

        let mut verifier_transcript = Transcript::new(b"sumcheck-test");
        let result = verify(
            claimed_sum,
            poly.variable_count(),
            &proof,
            &mut verifier_transcript,
            oracle_for(&poly),
        );

        assert_eq!(result, Err(VerificationError::DegreeTooHigh { round: 0 }));
    }

    #[test]
    fn final_evaluation_mismatch_is_rejected() {
        let poly = test_mle();

        let mut prover_transcript = Transcript::new(b"sumcheck-test");
        let (claimed_sum, proof) = prove(&poly, &mut prover_transcript);

        let mut verifier_transcript = Transcript::new(b"sumcheck-test");

        let result = verify(
            claimed_sum,
            poly.variable_count(),
            &proof,
            &mut verifier_transcript,
            |_point| Fp::new(999), // deliberately incorrect oracle
        );

        assert_eq!(result, Err(VerificationError::FinalEvaluationMismatch));
    }

    #[test]
    fn mismatched_transcript_label_is_rejected() {
        let poly = test_mle();

        let mut prover_transcript = Transcript::new(b"sumcheck-test");
        let (claimed_sum, proof) = prove(&poly, &mut prover_transcript);

        let mut verifier_transcript = Transcript::new(b"different-label");
        let result = verify(
            claimed_sum,
            poly.variable_count(),
            &proof,
            &mut verifier_transcript,
            oracle_for(&poly),
        );

        assert!(result.is_err());
    }
}
