use std::{ fmt::Display};
use rand::rngs::OsRng;
use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce, aead::{Aead}
};
use base64::{Engine};
use rand::{prelude::ThreadRng, RngCore};
use rsa::{
    pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey},
    pkcs1v15::{Signature, SigningKey, VerifyingKey},
    sha2::Sha256,
    signature::{Signer, Verifier},
    Pkcs1v15Encrypt, PublicKey, RsaPrivateKey, RsaPublicKey,
};
use thiserror::Error;

enum E2EType {
    Sender,
    Reciever,
}

pub struct E2EOfflineBuilder {
    rng: OsRng,
    reciever_public_key: Option<RsaPublicKey>,
    sender_public_key: Option<RsaPublicKey>,

    my_private_key: RsaPrivateKey,

    shared_key: Option<Vec<u8>>,
    aes: Option<Aes256Gcm>,

    my_type: E2EType,
}


impl E2EOfflineBuilder {
    pub fn new_sender() -> Self {
        let mut rng = OsRng;
        let my_private_key = RsaPrivateKey::new(&mut rng, 1024).unwrap();

        let shared_key = Aes256Gcm::generate_key(&mut rng);

        Self {
            rng,
            reciever_public_key: None,
            sender_public_key: Some(RsaPublicKey::from(&my_private_key)),
            my_private_key,
            shared_key: Some((&shared_key[..]).into()),
            my_type: E2EType::Sender,
            aes: Some(Aes256Gcm::new(&shared_key)),
        }
    }

    pub fn new_reciever() -> Self {
        let mut rng = rand::rngs::OsRng;


        let my_private_key = RsaPrivateKey::new(&mut rng, 1024).unwrap();

        Self {
            rng,
            reciever_public_key: Some(RsaPublicKey::from(&my_private_key)),
            sender_public_key: None,
            my_private_key,
            shared_key: None,
            aes: None,
            my_type: E2EType::Reciever,
        }
    }

    pub fn set_other_public_key_encoded(&mut self, pubkey: &str) -> anyhow::Result<()> {
        let pubkey = pubkey.trim();

        let der = base64::engine::general_purpose::URL_SAFE.decode(pubkey).expect("Failed to decode pubkey: ");

        let pubkey = RsaPublicKey::from_pkcs1_der(&der)?;

        self.set_other_public_key(pubkey);

        Ok(())
    }

     fn set_other_public_key(&mut self, pubkey: RsaPublicKey) {
        match self.my_type {
            E2EType::Sender => self.reciever_public_key.replace(pubkey),
            E2EType::Reciever => self.sender_public_key.replace(pubkey),
        };
    }

    fn get_pubkey(&self) -> &RsaPublicKey {
        match self.my_type {
            E2EType::Sender => &self.sender_public_key.as_ref().unwrap(),
            E2EType::Reciever => &self.reciever_public_key.as_ref().unwrap(),
        }
    }

    pub fn get_pubkey_encoded(&self) -> anyhow::Result<String>  {
        let der = self.get_pubkey().to_pkcs1_der()?;

        Ok(base64::engine::general_purpose::URL_SAFE.encode(der.as_bytes()))
    }



    pub fn send(&mut self) -> anyhow::Result<String>  {
        let signing_key = SigningKey::<Sha256>::new(self.my_private_key.clone());

        let shared_key_encrypted = self.reciever_public_key.as_ref().unwrap().encrypt(
            &mut self.rng,
            Pkcs1v15Encrypt,
            &self.shared_key.as_ref().unwrap()[..],
        )?;

        let ske_encoded = base64::engine::general_purpose::URL_SAFE.encode(shared_key_encrypted);


        let signature = signing_key.sign(ske_encoded.as_bytes());
        let signature_encoded = base64::engine::general_purpose::URL_SAFE.encode(signature);

        Ok(format!("{ske_encoded}.{signature_encoded}"))
    }

    pub fn recieve(&mut self, message: &str) -> anyhow::Result<()> {
        let mut message = message.trim().split('.');

        let (ske_encoded, signature_encoded) = (
            message.next().ok_or(RecieveError::InvalidString)?,
            message.next().ok_or(RecieveError::InvalidString)?,
        );

        let signature = base64::engine::general_purpose::URL_SAFE.decode(signature_encoded)?;

        let verifier =
            VerifyingKey::<Sha256>::from(self.sender_public_key.as_ref().unwrap().clone());

        if verifier
            .verify(
                ske_encoded.as_bytes(),
                &Signature::from(signature.into_boxed_slice()),
            )
            .is_err()
        {
            Err(RecieveError::FailedSignatureCheck)?;
        }

        

        let shared_key_encrypted = base64::engine::general_purpose::URL_SAFE.decode(ske_encoded)?;

        let shared_key = self
            .my_private_key
            .decrypt(Pkcs1v15Encrypt, &shared_key_encrypted[..])?;

        self.aes.replace(Aes256Gcm::new_from_slice(&shared_key)?);

        self.shared_key.replace(shared_key);

        Ok(())
    }

    pub fn get_shared_key(&self) -> anyhow::Result<String> {
        Ok(base64::engine::general_purpose::URL_SAFE.encode(self.shared_key.as_ref().unwrap()))
    }

    pub fn build(self) -> anyhow::Result<E2EOffline> {
        Ok(E2EOffline {
            aes: self.aes.unwrap(),
            rng: self.rng,
        })
    }
}

pub struct E2EOffline {
    aes: Aes256Gcm,
    rng: OsRng
}

impl E2EOffline {
    pub fn from_key_base64(key: &str) -> anyhow::Result<E2EOffline> {
        let rng = OsRng;

        let key = base64::engine::general_purpose::URL_SAFE.decode(key.trim())?;
        let aes = Aes256Gcm::new_from_slice(&key)?;

        Ok(Self {
            aes,
            rng
        })
    }

    pub fn encrypt(&mut self, plaintext: &str) ->  anyhow::Result<String> {
        let mut nonce = [0u8; 12];
        self.rng.fill_bytes(&mut nonce);

        let nonce = Nonce::from_mut_slice(&mut nonce);
        let nonce_encoded = base64::engine::general_purpose::URL_SAFE.encode(&nonce);

        let ciphertext = self
            .aes
            .encrypt(nonce, plaintext.as_bytes())
            .unwrap();

        let ciphertext = base64::engine::general_purpose::URL_SAFE.encode(ciphertext);

        Ok(format!("{nonce_encoded}.{ciphertext}"))
    }

    pub fn decrypt(&mut self, ciphertext: &str) -> anyhow::Result<String> {
        let mut ciphertext = ciphertext.trim().split(".");
        let (nonce, ciphertext) = (
            ciphertext.next().ok_or(RecieveError::InvalidString)?,
            ciphertext.next().ok_or(RecieveError::InvalidString)?,
        );

        let nonce = base64::engine::general_purpose::URL_SAFE.decode(nonce)?;
        let nonce = Nonce::from_slice(&nonce);

        let ciphertext = base64::engine::general_purpose::URL_SAFE.decode(ciphertext)?;

        let result = self
            .aes
            .decrypt(nonce, &ciphertext[..])
            .unwrap();

        let s = String::from_utf8(result)?;
        Ok(s)
    }
}

#[derive(Error, Debug)]
enum RecieveError {
    InvalidString,
    FailedSignatureCheck,
}

impl Display for RecieveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecieveError::InvalidString => write!(f, "invalid sender string"),
            RecieveError::FailedSignatureCheck => write!(f, "signature failed"),
        }
    }
}

//new sender                 //new recieve

//
