use num::traits::{CheckedAdd, CheckedSub, Zero};
use std::collections::BTreeMap;

pub trait Config: crate::system::Config {
	type Balance: Zero + CheckedSub + CheckedAdd + Copy;
}

#[derive(Debug)]
pub struct Pallet<T: Config> {
	balances: BTreeMap<T::AccountId, T::Balance>,
}

impl<T: Config> Pallet<T> {
	pub fn new() -> Self {
		Self { balances: BTreeMap::new() }
	}

	pub fn set_balance(&mut self, who: &T::AccountId, amount: T::Balance) {
		self.balances.insert(who.clone(), amount);
	}

	pub fn balance(&self, who: &T::AccountId) -> T::Balance {
		*self.balances.get(who).unwrap_or(&T::Balance::zero())
	}

	pub fn transfer(
		&mut self,
		caller: &T::AccountId,
		to: &T::AccountId,
		amount: T::Balance,
	) -> crate::support::DispatchResult {
		let caller_balance: T::Balance = self.balance(caller);
		let to_balance: T::Balance = self.balance(to);

		let new_caller_balance = caller_balance.checked_sub(&amount).ok_or("Not enough funds.")?;
		let new_to_balance = to_balance.checked_add(&amount).ok_or("Overflow")?;

		self.balances.insert(caller.clone(), new_caller_balance);
		self.balances.insert(to.clone(), new_to_balance);

		Ok(())
	}
}

pub enum Call<T: Config> {
	Transfer { to: T::AccountId, amount: T::Balance },
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
			Call::Transfer { to, amount } => {
				self.transfer(&caller, &to, amount)?;
			},
		}
		Ok(())
	}
}
#[cfg(test)]
mod tests {
	struct TestConfig;

	impl crate::system::Config for TestConfig {
		type AccountId = String;
		type BlockNumber = u32;
		type Nonce = u32;
	}

	impl super::Config for TestConfig {
		type Balance = u128;
	}

	#[test]
	fn init_balances() {
		let mut balances = super::Pallet::<TestConfig>::new();

		let ALICE = "alice".to_string();
		let BOB = "bob".to_string();

		assert_eq!(balances.balance(&ALICE), 0);
		balances.set_balance(&ALICE, 100);
		assert_eq!(balances.balance(&ALICE), 100);
		assert_eq!(balances.balance(&BOB), 0);
	}

	#[test]
	fn transfer_balance() {
		let mut balances = super::Pallet::<TestConfig>::new();

		let ALICE = "alice".to_string();
		let BOB = "bob".to_string();

		assert_eq!(balances.transfer(&ALICE, &BOB, 51), Err("Not enough funds."));

		balances.set_balance(&ALICE, 100);
		assert_eq!(balances.transfer(&ALICE, &BOB, 51), Ok(()));
		assert_eq!(balances.balance(&ALICE), 49);
		assert_eq!(balances.balance(&BOB), 51);

		assert_eq!(balances.transfer(&ALICE, &BOB, 51), Err("Not enough funds."));
	}
}
