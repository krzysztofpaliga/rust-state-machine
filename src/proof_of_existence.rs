use core::fmt::Debug;
use std::collections::BTreeMap;
use crate::support::DispatchResult;

pub trait Config: crate::system::Config {
    type Content: Debug + Ord;
}

#[derive(Debug)]
pub struct Pallet<T: Config> {
    claims: BTreeMap<T::Content, T::AccountId>,
}

impl<T: Config> Pallet<T> {
    pub fn new() -> Self {
        Self { claims: BTreeMap::new() }
    }

    pub fn get_claim(&self, claim: &T::Content) -> Option<&T::AccountId> {
        self.claims.get(claim)
    }

    pub fn create_claim(&mut self, caller: T::AccountId, claim: T::Content) -> DispatchResult {
        if self.claims.contains_key(&claim) {
            return Err(&"this content is already claimed");
        }
        self.claims.insert(claim, caller);
        Ok(())
    }

    pub fn revoke_claim(&mut self, caller: T::AccountId, claim: T::Content) -> DispatchResult {
        let claim_owner = self.get_claim(&claim).ok_or("claim does not exist")?;
        if caller != *claim_owner {
            return Err("this content is owned by someone else");
        }
        self.claims.remove(&claim);
        Ok(())
    }
}

pub enum Call<T: Config> {
    CreateClaim { content: T::Content },
    RevokeClaim { content: T::Content },
}

impl<T: Config> crate::support::Dispatch for Pallet<T> {
    type Caller = T::AccountId;
    type Call = Call<T>;

    fn dispatch(
        &mut self,
        caller: Self::Caller,
        call: Self::Call,
    ) -> crate::support::DispatchResult {
        match call {
            Call::CreateClaim { content } => {
                self.create_claim(caller, content)?;
            },
            Call::RevokeClaim { content } => {
                self.revoke_claim(caller, content)?;
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    struct TestConfig;

    impl super::Config for TestConfig {
        type Content = &'static str;
    }

    impl crate::system::Config for TestConfig {
        type AccountId = &'static str;
        type BlockNumber = u32;
        type Nonce = u32;
    }

    #[test]
    fn basic_proof_of_existence() {
        let mut poe = super::Pallet::<TestConfig>::new();
        assert_eq!(poe.get_claim(&"Hello, world!"), None);
        assert_eq!(poe.create_claim("alice", "Hello, world!"), Ok(()));
        assert_eq!(poe.get_claim(&"Hello, world!"), Some(&"alice"));
        assert_eq!(
            poe.create_claim("bob", "Hello, world!"),
            Err("this content is already claimed")
        );
        assert_eq!(poe.revoke_claim("alice", "Hello, world!"), Ok(()));
        assert_eq!(poe.create_claim("bob", "Hello, world!"), Ok(()));
    }
}
