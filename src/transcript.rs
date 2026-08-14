use sha2::{Digest, Sha256};
use crate::field::Fp;

pub struct Transcript {
    hasher: Sha256,
}

impl Transcript {
    pub fn new(label: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(label);
        Transcript { hasher }
    }

    fn append_message(&mut self, label: &[u8], data: &[u8]) {
        self.hasher.update(label);
        self.hasher.update(data);
    }

    pub fn append_scalar(&mut self, label: &[u8], scalar: Fp) {
        self.append_message(label, &scalar.to_bytes());
    }

    pub fn append_scalars(&mut self, label: &[u8], scalars: &[Fp]) {
        self.append_message(label, &(scalars.len() as u64).to_le_bytes());
        for &sc in scalars {
            self.append_scalar(label, sc);
        }
    }

    pub fn challenge_scalar(&mut self, label: &[u8]) -> Fp {
        let mut peek = self.hasher.clone();
        peek.update(label);
        
        let digest = peek.finalize();

        self.hasher.update(label);
        self.hasher.update(&digest);

        Fp::from_bytes_reduce(&digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_appends_give_same_challenge() {
        let mut t1 = Transcript::new(b"test");
        let mut t2 = Transcript::new(b"test");
        t1.append_scalar(b"x", Fp::new(5));
        t2.append_scalar(b"x", Fp::new(5));
        assert_eq!(t1.challenge_scalar(b"challenge"), t2.challenge_scalar(b"challenge"));
    }

    #[test]
    fn different_appends_give_different_challenge() {
        let mut t1 = Transcript::new(b"test");
        let mut t2 = Transcript::new(b"test");
        t1.append_scalar(b"x", Fp::new(5));
        t2.append_scalar(b"x", Fp::new(6));
        assert_ne!(t1.challenge_scalar(b"challenge"), t2.challenge_scalar(b"challenge"));
    }

        #[test]
    fn different_labels_give_different_challenge() {
        let mut t1 = Transcript::new(b"test");
        let mut t2 = Transcript::new(b"test");
        t1.append_scalar(b"x", Fp::new(5));
        t2.append_scalar(b"y", Fp::new(5));
        assert_ne!(t1.challenge_scalar(b"challenge"), t2.challenge_scalar(b"challenge"));
    }

    #[test]
    fn different_append_order_gives_different_challenge() {
        let mut t1 = Transcript::new(b"test");
        let mut t2 = Transcript::new(b"test");
        t1.append_scalar(b"x", Fp::new(5));
        t1.append_scalar(b"y", Fp::new(6));
        t2.append_scalar(b"y", Fp::new(6));
        t2.append_scalar(b"x", Fp::new(5));
        assert_ne!(t1.challenge_scalar(b"challenge"), t2.challenge_scalar(b"challenge"));
    }

    #[test]
    fn challenge_changes_after_append() {
        let mut transcript = Transcript::new(b"test");
        transcript.append_scalar(b"x", Fp::new(5));
        let first = transcript.challenge_scalar(b"challenge");
        transcript.append_scalar(b"y", Fp::new(6));
        let second = transcript.challenge_scalar(b"challenge");
        assert_ne!(first, second);
    }

    #[test]
    fn consecutive_challenges_give_different_challenge() {
        let mut transcript = Transcript::new(b"test");
        transcript.append_scalar(b"x", Fp::new(5));
        let first = transcript.challenge_scalar(b"challenge");
        let second = transcript.challenge_scalar(b"challenge");
        assert_ne!(first, second);
    }

    #[test]
    fn scalar_vector_appends_give_different_challenge() {
        let mut t1 = Transcript::new(b"test");
        let mut t2 = Transcript::new(b"test");
        t1.append_scalars(b"x", &[Fp::new(5), Fp::new(6)]);
        t2.append_scalar(b"x", Fp::new(5));
        t2.append_scalar(b"x", Fp::new(6));
        assert_ne!(t1.challenge_scalar(b"challenge"), t2.challenge_scalar(b"challenge"));
    }
}