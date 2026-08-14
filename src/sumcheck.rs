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

    let mut p = poly.clone();
    for _ in 0..n_variables {
        let mid = p.evaluations.len() / 2;
        let mut eval_at_0 = Fp::zero();
        let mut eval_at_1 = Fp::zero();
        for i in 0..mid {
            eval_at_0 = eval_at_0 + p.evaluations[i];
            eval_at_1 = eval_at_1 + p.evaluations[mid + i];
        }

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

    for (round, (poly, &r)) in proof.round_polynomials.iter().zip(challenges).enumerate() {
        let round_sum = poly.evaluate(Fp::zero()) + poly.evaluate(Fp::one());
        if round_sum != expected {
            return Err(VerificationError::InconsistentRoundSum { round });
        }

        transcript.append_scalars(b"round_poly", &poly.coefficients);
        let r = transcript.challenge_scalar(b"challenge");
        
        expected = poly.evaluate(r);
    }

    if oracle(challenges) != expected {
        return Err(VerificationError::FinalEvaluationMismatch);
    }

    Ok(())
}
