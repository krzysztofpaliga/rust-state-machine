mod balances;
mod support;
mod system;
mod proof_of_existence;

use crate::support::Dispatch;

mod types {
	pub type AccountId = String;
	pub type Balance = u128;
	pub type BlockNumber = u32;
	pub type Nonce = u32;
	pub type Extrinsic = crate::support::Extrinsic<AccountId, crate::RuntimeCall>;
	pub type Header = crate::support::Header<BlockNumber>;
	pub type Block = crate::support::Block<Header, Extrinsic>;
    pub type Content = &'static str;
}

pub enum RuntimeCall {
	Balances(balances::Call<Runtime>),
    ProofOfExistence(proof_of_existence::Call<Runtime>),
}

#[derive(Debug)]
pub struct Runtime {
	system: system::Pallet<Self>,
	balances: balances::Pallet<Self>,
    proof_of_existence: proof_of_existence::Pallet<Self>,
}

impl system::Config for Runtime {
	type AccountId = types::AccountId;
	type BlockNumber = types::BlockNumber;
	type Nonce = types::Nonce;
}

impl balances::Config for Runtime {
	type Balance = types::Balance;
}

impl proof_of_existence::Config for Runtime {
    type Content = types::Content;
}

impl Runtime {
	fn new() -> Self {
		Self { system: system::Pallet::new(), balances: balances::Pallet::new(), proof_of_existence: proof_of_existence::Pallet::new() }
	}
	fn execute_block(&mut self, block: types::Block) -> support::DispatchResult {
		self.system.inc_block_number();
		if block.header.block_number != self.system.block_number() {
			return Err("block number does not match what is expected")
		}

		for (i, support::Extrinsic { caller, call }) in block.extrinsics.into_iter().enumerate() {
			self.system.inc_nonce(&caller);
			let _res = self.dispatch(caller, call).map_err(|e| {
				eprintln!(
					"Extrinsic Error\n\tBlock Number: {}\n\tExtrinsic Number: {}\n\tError: {}",
					block.header.block_number, i, e
				)
			});
		}
		Ok(())
	}
}

impl crate::support::Dispatch for Runtime {
	type Caller = <Runtime as system::Config>::AccountId;
	type Call = RuntimeCall;

	fn dispatch(
		&mut self,
		caller: Self::Caller,
		runtime_call: Self::Call,
	) -> support::DispatchResult {
		match runtime_call {
			RuntimeCall::Balances(call) => {
				self.balances.dispatch(caller, call)?;
			},
            RuntimeCall::ProofOfExistence(call) => {
                self.proof_of_existence.dispatch(caller, call)?;
            }
		}
		Ok(())
	}
}

fn main() {
	let ALICE = "alice".to_string();
	let BOB = "bob".to_string();
	let CHARLIE = "charlie".to_string();

	let mut runtime = Runtime::new();

	runtime.balances.set_balance(&ALICE, 100);

	assert_eq!(runtime.system.block_number(), 0);

	let block_1 = types::Block {
		header: support::Header { block_number: 1 },
		extrinsics: vec![
			support::Extrinsic {
				caller: ALICE.clone(),
				call: RuntimeCall::Balances(balances::Call::Transfer {
					to: BOB.clone(),
					amount: 30,
				}),
			},
			support::Extrinsic {
				caller: ALICE.clone(),
				call: RuntimeCall::Balances(balances::Call::Transfer {
					to: CHARLIE.clone(),
					amount: 20,
				}),
			},
		],
	};

	runtime.execute_block(block_1).expect("invalid block");

	assert_eq!(runtime.balances.balance(&ALICE), 50);
	assert_eq!(runtime.balances.balance(&BOB), 30);
	assert_eq!(runtime.balances.balance(&CHARLIE), 20);

    let block_2 = types::Block {
        header: support::Header { block_number: 2 },
        extrinsics: vec![
            support::Extrinsic {
                caller: ALICE.clone(),
                call: RuntimeCall::ProofOfExistence(proof_of_existence::Call::CreateClaim {
                    content: "Hello, world!",
                }),
            },
            support::Extrinsic {
                caller: BOB.clone(),
                call: RuntimeCall::ProofOfExistence(proof_of_existence::Call::CreateClaim {
                    content: "Hello, world!",
                }),
            },
        ],
    };

	runtime.execute_block(block_2).expect("invalid block");


    assert_eq!(runtime.proof_of_existence.get_claim(&"Hello, world!"), Some(&ALICE));
    assert_eq!(runtime.proof_of_existence.get_claim(&"Hello, world!!"), None);

    let block_3 = types::Block {
        header: support::Header { block_number: 3 },
        extrinsics: vec![
            support::Extrinsic {
                caller: ALICE.clone(),
                call: RuntimeCall::ProofOfExistence(proof_of_existence::Call::RevokeClaim {
                    content: "Hello, world!",
                }),
            },
            support::Extrinsic {
                caller: BOB.clone(),
                call: RuntimeCall::ProofOfExistence(proof_of_existence::Call::CreateClaim {
                    content: "Hello, world!",
                }),
            },
        ],
    };

	runtime.execute_block(block_3).expect("invalid block");


    assert_eq!(runtime.proof_of_existence.get_claim(&"Hello, world!"), Some(&BOB));
    assert_eq!(runtime.proof_of_existence.get_claim(&"Hello, world!!"), None);

	println!("{:#?}", runtime);
}
