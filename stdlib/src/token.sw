library token;
//! Functionality for performing common operations on tokens.

use ::ops::*;
use ::address::Address;

const OUTPUT_LENGTH_LOCATION = 384;


/// Mint `n` coins of the current contract's token_id.
pub fn mint(n: u64) {
    asm(r1: n) {
        mint r1;
    }
}

/// Burn `n` coins of the current contract's token_id.
pub fn burn(n: u64) {
    asm(r1: n) {
        burn r1;
    }
}

/// Transfer amount `coins` of type `token_id` to address `recipient`.
pub fn transfer_to_output(coins: u64, token_id: b256, recipient: Address) {
    // get length of outputs from TransactionScript outputsCount (7th word):
    // @todo tx format may change, in which case the magic number "384" must be changed.
    // @todo make this a const.
    let length: u8 = asm(outputs_length, outputs_length_ptr: OUTPUT_LENGTH_LOCATION) {
        lw outputs_length outputs_length_ptr;
        outputs_length: u8
    };
    // see spec for output types: https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/compressed_tx_format.md#output
    let target_type = 4;
    let mut index: u8 = 0;
    let mut outputIndex = 0;

    // check if `type` matches target type:
    // @todo fix this once issue #440 is fixed. remove asm block
    // while index < length
    while asm(i: index, l: length, res) {
        lt res i l;
        res: bool
    } {
        let type_match = asm(slot: index, type, target: 4, bytes: 8, res) {
            xos t slot;
            meq res type target bytes;
            res: bool
        };

        if type_match {
            // check if `amount` is zero:
            let amount_is_zero = asm(slot: index, a, amount_ptr, output, is_zero, bytes: 8) {
                xos output slot;
                addi amount_ptr output i64;
                lw a amount_ptr;
                meq is_zero a zero bytes;
                is_zero: bool
            };
            if amount_is_zero {
                outputIndex = index;
            } else {
                // index = index + 1;

                //@todo fix this once issue #440 is fixed. remove asm block
                index = asm(i: index, res) {
                    add res i one;
                    res: u8
                };
            }
        } else {
            // index = index + 1;
            // @todo fix this once issue #440 is fixed. remove asm block
                index = asm(i: index, res) {
                    add res i one;
                    res: u8
                };
        }
    }
    asm(amount: coins, id: token_id, recipient, output: index) {
        tro recipient output amount id;
    }
    // @todo should revert if there are no free outputs.
}

/// !!! UNCONDITIONAL transfer of amount `coins` of type `token_id` to contract at `contract_id`.
/// This will allow the transfer of coins even if there is no way to retrieve them !!!
/// Use of this function can lead to irretrievable loss of coins if not used with caution.
// @todo use type `ContractId` if implemented.
pub fn force_transfer(coins: u64, token_id: b256, contract_id: b256) {
    asm(coins, token_id, contract_id) {
        tr contract_id coins token_id;
    }
}
