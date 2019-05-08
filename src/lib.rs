#![cfg_attr(not(any(test, feature = "std")), no_std)]

use ink_core::{
    memory::format,
    storage,
    env::{
        self,
        AccountId,
        Balance,
    },
};
use ink_lang::contract;
use parity_codec::{
    Decode,
    Encode,
};

#[derive(Encode, Decode)]
enum Event{
    PreFund{
        total_fund: u128,
        your_balance: u128,
        opposit_balance:u128,
    },
    PostFund{
        total_fund: u128,
        your_balance: u128,
        opposit_balance:u128,

    },
    FinalState{
        total_fund: u128,
        your_balance: u128,
        opposit_balance:u128,
        your_pos: u32,
        opposit_pos: u32,
    },
    SaveState{

    },
}
fn deposit_event(event: Event) {
    env::deposit_raw_event(&event.encode()[..])
}

contract! {
    struct StateChannel {
        channel_id: storage::Value<u32>,
        nonce: storage::Value<u32>,
        total_fund: storage::Value<u128>,
        balances: storage::HashMap<AccountId, u128>,
        positions: storage::HashMap<AccountId, u32>,

        proposal_channel_id: storage::HashMap<AccountId,u32>,
        proposal_total_fund: storage::HashMap<AccountId,u128>,
        proposal_balances: storage::HashMap<(AccountId,AccountId), u128>,
        proposal_positions: storage::HashMap<(AccountId,AccountId), u32>,

        proposals_num: storage::Value<u32>,
        game_state: storage::Value<u32>,  // 0 - start 1 - prefund 2 - post 3 - final

    }

    impl Deploy for StateChannel {
        fn deploy(&mut self) {
           env.println("Start deploying");

            self.channel_id.set(1);
            self.nonce.set(0);
            self.total_fund.set(0);
            self.proposals_num.set(0);
            self.game_state.set(0);
            env.println("Deployed");

        }
    }

    impl StateChannel {
        pub(external) fn test(&mut self) -> bool{
         env.println("Test");
         return true;
        }
        pub(external) fn send_prefund_state(&mut self,channel_id: u32, total_fund: u128, your_balance: u128, opposit_balance: u128, opposit_account: AccountId) -> bool{
            env.println("Prefund state received");

            let caller = env.caller();
            env.println(&format!("Balances: total:{:?}, your:{:?}, opposit:{:?} ", total_fund , your_balance, opposit_balance));
            self.validate_proposal(channel_id,total_fund, your_balance, opposit_balance);
            self.proposals_num.set(self.proposals_num.get()+1);
            self.check_game_state(1);
            self.proposal_channel_id.insert(caller, channel_id);
            self.proposal_total_fund.insert(caller, total_fund);

            self.proposal_balances.insert((caller, caller), your_balance);
            self.proposal_balances.insert((caller, opposit_account), opposit_balance);
            self.game_state.set(1);

            env.println("Saving prefund proposal done");
            deposit_event(Event::PreFund{total_fund, your_balance, opposit_balance});
            if *self.proposals_num.get() == 2 {
                env.println("2 required  prefund proposals received");
                self.proposals_num.set(0);
                let valid = self.validate_pre_and_post_proposals(&opposit_account);
                env.println("All prefund Proposals correct");

                if valid == true{
                    self.save_pre_and_post_state(&opposit_account);
                }
                return valid;

            }else{
                env.println(" 1 prefund proposals received ");

                return true;
            }

        }

        pub(external) fn send_postfund_state(&mut self, channel_id: u32, total_fund: u128, your_balance: u128, opposit_balance: u128, opposit_account: AccountId)-> bool{
            env.println("Postfund state received");

            let caller = env.caller();
            env.println(&format!("Balances: total:{:?}, your:{:?}, opposit:{:?} ", total_fund , your_balance, opposit_balance));

            self.validate_proposal(channel_id,total_fund, your_balance, opposit_balance);
            env.println("Postfund Proposal correct");

            self.proposals_num.set(*self.proposals_num.get()+1);
            self.check_game_state(2);
            self.proposal_channel_id.insert(caller, channel_id);
            self.proposal_total_fund.insert(caller, total_fund);

            self.proposal_balances.insert((caller, caller), your_balance);
            self.proposal_balances.insert((caller, opposit_account), opposit_balance);
            self.game_state.set(2);
             env.println("Saving fund post proposal done");

            deposit_event(Event::PostFund{total_fund, your_balance, opposit_balance});
            if *self.proposals_num.get() == 2{
                env.println("2 required  prefund proposals received ");

                self.proposals_num.set(0);

                let valid = self.validate_pre_and_post_proposals(&opposit_account);
                if valid==true{
                    self.save_pre_and_post_state(&opposit_account);
                }

                return valid;

            }else{
                env.println("1 prefund proposal received ");

                return true;
            }
        }

        pub(external) fn send_final_state(&mut self,channel_id: u32,total_fund: u128, your_balance: u128, opposit_balance: u128, opposit_account: AccountId, your_pos: u32, opposit_pos: u32) -> bool{
            env.println("Final state received");

            let caller = env.caller();
            env.println(&format!("Balances: total:{:?}, your:{:?}, opposit{:?} ", total_fund , your_balance, opposit_balance));

            self.validate_proposal(channel_id,total_fund, your_balance, opposit_balance);

            self.proposals_num.set(*self.proposals_num.get()+1);
            self.check_game_state(3);
            self.proposal_channel_id.insert(caller, channel_id);
            self.proposal_total_fund.insert(caller, total_fund);

            self.proposal_balances.insert((caller, caller), your_balance);
            self.proposal_balances.insert((caller, opposit_account), opposit_balance);


            self.proposal_positions.insert((caller, caller), your_pos);
            self.proposal_positions.insert((caller, opposit_account), opposit_pos);
            self.game_state.set(3);
            env.println("Saving fund final proposal done");
            deposit_event(Event::FinalState{total_fund, your_balance, opposit_balance, your_pos, opposit_pos});

            if *self.proposals_num.get() == 2{
                env.println("2 reuquired final proposals received");

                self.proposals_num.set(0);
                self.check_game_logic(your_balance, opposit_balance, your_pos, opposit_pos);
                let valid = self.validate_final_proposals(&opposit_account);
                if valid == true{
                    self.game_state.set(0);
                    self.save_final_state(&opposit_account);
                }
                return valid;

            }else{
                env.println("1 final proposals received");
                return true;
            }
        }

        pub(external) fn clear_all_data(&mut self){
            self.total_fund.set(0);
            self.proposals_num.set(0);
            self.game_state.set(0);
            env.println("Recovered to initial state");

        }
    }

    impl StateChannel{
        fn validate_pre_and_post_proposals(&mut self,opposit_account: &AccountId) -> bool{
            env::println("Proposals validation");
            let caller = &(env::caller());
            env::println(&format!("assert total fund your:{:?}, opposit{:?} ", *self.proposal_total_fund.get(caller).unwrap(), *self.proposal_total_fund.get(opposit_account).unwrap()));
            assert!(*self.proposal_total_fund.get(caller).unwrap() == *self.proposal_total_fund.get(opposit_account).unwrap());

            env::println(&format!("assert balances for caller your:{:?}, opposit{:?} ", *self.proposal_balances.get(&(*caller, *caller)).unwrap(), *self.proposal_balances.get(&(*opposit_account, *caller)).unwrap()));
            assert!(*self.proposal_balances.get(&(*caller, *caller)).unwrap() == *self.proposal_balances.get(&(*opposit_account, *caller)).unwrap());
            env::println(&format!("assert balances for opposit your:{:?}, opposit{:?} ", *self.proposal_balances.get(&(*caller,*opposit_account)).unwrap(),  *self.proposal_balances.get(&(*opposit_account, *opposit_account)).unwrap()));
            let res = *self.proposal_balances.get(&(*caller, *opposit_account)).unwrap() == *self.proposal_balances.get(&(*opposit_account, *opposit_account)).unwrap();
            env::println(&format!("bool res:{:?}",res));

            assert!(res);
            env::println("Proposals validated");
            return true;

        }

        fn validate_final_proposals(&mut self,opposit_account: &AccountId) -> bool{
            env::println("Proposals validation");
            let caller = &(env::caller());
            env::println(&format!("assert total fund your:{:?}, opposit: {:?} ", *self.proposal_total_fund.get(caller).unwrap(), *self.proposal_total_fund.get(opposit_account).unwrap()));
            assert!(*self.proposal_total_fund.get(caller).unwrap() == *self.proposal_total_fund.get(opposit_account).unwrap());

            env::println(&format!("assert balances for caller your:{:?}, opposit{:?} ", *self.proposal_balances.get(&(*caller, *caller)).unwrap(), *self.proposal_balances.get(&(*opposit_account, *caller)).unwrap()));
            assert!(*self.proposal_balances.get(&(*caller, *caller)).unwrap() == *self.proposal_balances.get(&(*opposit_account, *caller)).unwrap());
            env::println(&format!("assert balances for opposit your:{:?}, opposit{:?} ", *self.proposal_balances.get(&(*caller,*opposit_account)).unwrap(),  *self.proposal_balances.get(&(*opposit_account, *opposit_account)).unwrap()));
            let res = *self.proposal_balances.get(&(*caller, *opposit_account)).unwrap() == *self.proposal_balances.get(&(*opposit_account, *opposit_account)).unwrap();
            env::println(&format!("bool res:{:?}",res));

            assert!(res);
            env::println(&format!("assert positions for oppost your:{:?}, opposit{:?} ", *self.proposal_positions.get(&(*caller, *opposit_account)).unwrap(), *self.proposal_positions.get(&(*opposit_account, *opposit_account)).unwrap()));
            assert!(*self.proposal_positions.get(&(*caller, *opposit_account)).unwrap() == *self.proposal_positions.get(&(*opposit_account, *opposit_account)).unwrap());

            env::println(&format!("assert positions for caller your:{:?}, opposit{:?} ", *self.proposal_positions.get(&(*caller, *caller)).unwrap(), *self.proposal_positions.get(&(*opposit_account, *caller)).unwrap()));
            assert!(*self.proposal_positions.get(&(*caller, *caller)).unwrap() == *self.proposal_positions.get(&(*opposit_account, *caller)).unwrap());
            env::println("proposals validated");

            return true;

        }


        fn save_pre_and_post_state(&mut self, opposite_account: &AccountId) -> bool{
             env::println("Saving state");
             let caller = &(env::caller());

              self.total_fund.set(*self.proposal_total_fund.get(caller).unwrap());

              self.balances.insert(*opposite_account, *self.proposal_balances.get(&(*caller, *opposite_account)).unwrap());
              self.balances.insert(*caller,*self.proposal_balances.get(&(*caller, *caller)).unwrap());

              deposit_event(Event::SaveState{});
              env::println("state saved");

              return true;
        }

        fn save_final_state(&mut self, opposite_account: &AccountId) -> bool{
             env::println("Saving state");
             let caller = &(env::caller());

              self.total_fund.set(*self.proposal_total_fund.get(caller).unwrap());

              self.balances.insert(*opposite_account, *self.proposal_balances.get(&(*caller, *opposite_account)).unwrap());
              self.balances.insert(*caller,*self.proposal_balances.get(&(*caller, *caller)).unwrap());

              self.positions.insert(*caller,*self.proposal_positions.get(&(*caller,*caller)).unwrap());
              self.positions.insert(*opposite_account,*self.proposal_positions.get(&(*caller,*opposite_account)).unwrap());
              deposit_event(Event::SaveState{});
              env::println("state saved");

              return true;
        }

        fn validate_proposal(&mut self,channel_id: u32,total_fund: u128, your_balance: u128, opposit_balance: u128) -> bool{
               env::println("Proposal validation");
               env::println(&format!("channel_id_cur:{:?}, channel_id_sent:{:?}", *self.channel_id.get(), channel_id));
               assert!(*self.channel_id.get() == channel_id);
               env::println(&format!("y:{:?}, o:{:?}", your_balance, opposit_balance));
               let sum = your_balance + opposit_balance;
               env::println(&format!("total:{:?}, calculated total:{:?}", total_fund, sum));
               assert!(total_fund ==  sum);
               env::println("Proposal validated");
               return true;
        }
        fn check_game_logic(&mut self, your_balance: u128, opposit_balance: u128, your_pos: u32, opposit_pos: u32) -> bool{
            env::println("Game logic checking");
            env::println(&format!("positions: your {:?}, opposit:{:?}", your_pos, opposit_pos));

            if your_pos>opposit_pos {
                env::println("Check your balance larger than opposite");
                assert!(your_balance > opposit_balance);
            }else{
                env::println("Check your balance smaller than opposite");
                assert!(your_balance < opposit_balance);
            }
            env::println("Game logic checked");
            return true;


        }

        fn check_game_state(&mut self, cur_game_state:u32) -> bool{
            env::println("Checking game state");
            env::println(&format!("provided game state: {:?}, contract game state: {:?} ", cur_game_state,*self.game_state.get()));
            if *self.proposals_num.get() == 2 {
                assert!(cur_game_state == *self.game_state.get());
            }else{
                assert!(cur_game_state > *self.game_state.get());
            }
            return true;
        }




    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn it_works() {
        let mut contract = StateChannel::deploy_mock();
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let bob = AccountId::try_from([0x1; 32]).unwrap();

        assert!(contract.send_prefund_state(1, 2, 1, 1, alice));
        assert!(contract.send_prefund_state(1, 2, 1, 1, alice));

        assert!(contract.send_postfund_state(1, 2, 1, 1, alice));
        assert!(contract.send_postfund_state(1, 2, 1, 1, alice));


        assert!(contract.send_final_state(1, 2, 1, 1, alice,1,2));
        assert!(contract.send_final_state(1, 2, 1, 1, alice,1,2));
    }
}
