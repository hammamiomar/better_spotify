pub mod pkce{
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use sha2::{Digest,Sha256};

    pub fn generate_code_verifier() -> String{
        let mut rng = thread_rng();
        let verifier : String = std::iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(128)
            .collect();
        verifier
    }

    pub fn generate_code_challenge(verifier: &str) -> String{
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        URL_SAFE_NO_PAD.encode(hash)
    }
}