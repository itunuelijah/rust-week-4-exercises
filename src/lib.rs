use std::str::FromStr;
use thiserror::Error;

// Custom errors for Bitcoin operations
#[derive(Error, Debug)]
pub enum BitcoinError {
    #[error("Invalid transaction format")]
    InvalidTransaction,
    #[error("Invalid script format")]
    InvalidScript,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Parse error: {0}")]
    ParseError(String),
}

// Generic Point struct for Bitcoin addresses or coordinates
#[derive(Debug, Clone, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        // TODO: Implement constructor for Point
        Self { x, y }
    }
}

// Custom serialization for Bitcoin transaction
pub trait BitcoinSerialize {
    fn serialize(&self) -> Vec<u8> {
        // TODO: Implement serialization to bytes
        Vec::new()
    }
}

// Legacy Bitcoin transaction
#[derive(Debug, Clone)]
pub struct LegacyTransaction {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl LegacyTransaction {
    pub fn builder() -> LegacyTransactionBuilder {
        // TODO: Return a new builder for constructing a transaction
        LegacyTransactionBuilder::new()
    }
}

// Transaction builder
pub struct LegacyTransactionBuilder {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl Default for LegacyTransactionBuilder {
    fn default() -> Self {
        // TODO: Implement default values
        Self {
            version: 1,
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time: 0,
        }
    }
}

impl LegacyTransactionBuilder {
    pub fn new() -> Self {
        // TODO: Initialize new builder by calling default
        Self::default()
    }

    pub fn version(mut self, version: i32) -> Self {
        // TODO: Set the transaction version
        self.version = version;
        self
    }

    pub fn add_input(mut self, input: TxInput) -> Self {
        // TODO: Add input to the transaction
        self.inputs.push(input);
        self
    }

    pub fn add_output(mut self, output: TxOutput) -> Self {
        // TODO: Add output to the transaction
        self.outputs.push(output);
        self
    }

    pub fn lock_time(mut self, lock_time: u32) -> Self {
        // TODO: Set lock_time for transaction
        self.lock_time = lock_time;
        self
    }

    pub fn build(self) -> LegacyTransaction {
        // TODO: Build and return the final LegacyTransaction
        LegacyTransaction {
            version: self.version,
            inputs: self.inputs,
            outputs: self.outputs,
            lock_time: self.lock_time,
        }
    }
}

// Transaction components
#[derive(Debug, Clone)]
pub struct TxInput {
    pub previous_output: OutPoint,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug, Clone)]
pub struct TxOutput {
    pub value: u64, // in satoshis
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct OutPoint {
    pub txid: [u8; 32],
    pub vout: u32,
}

// Simple CLI argument parser
pub fn parse_cli_args(args: &[String]) -> Result<CliCommand, BitcoinError> {
    // TODO: Match args to "send" or "balance" commands and parse required arguments
    match args {
        [cmd, amount, address] if cmd == "send" => {
            let amount = u64::from_str(amount)
                .map_err(|_| BitcoinError::ParseError("Invalid amount".into()))?;

            Ok(CliCommand::Send {
                amount,
                address: address.clone(),
            })
        }

        [cmd] if cmd == "balance" => Ok(CliCommand::Balance),

        _ => Err(BitcoinError::ParseError("Invalid command".into())),
    }
}

pub enum CliCommand {
    Send { amount: u64, address: String },
    Balance,
}

// Decoding legacy transaction
impl TryFrom<&[u8]> for LegacyTransaction {
    type Error = BitcoinError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: Parse binary data into a LegacyTransaction
        // Minimum length is 10 bytes (4 version + 4 inputs count + 4 lock_time)
        if data.len() < 10 {
            return Err(BitcoinError::InvalidTransaction);
        }
        let mut position = 0;

        fn read_bytes(data: &[u8], position: &mut usize, length: usize) -> Vec<u8> {
            let end = (*position + length).min(data.len());
            let mut result = vec![0u8; length];
            if *position < data.len() {
                let available_bytes = &data[*position..end];
                result[..available_bytes.len()].copy_from_slice(available_bytes);
            }
            *position = (*position + length).min(data.len());
            result
        }

        let version = i32::from_le_bytes(read_bytes(data, &mut position, 4).try_into().unwrap());

        let input_count =
            u32::from_le_bytes(read_bytes(data, &mut position, 4).try_into().unwrap());
        let mut inputs = Vec::with_capacity(input_count as usize);
        for _ in 0..input_count {
            let mut previous_txid = [0u8; 32];
            previous_txid.copy_from_slice(&read_bytes(data, &mut position, 32));

            let previous_vout =
                u32::from_le_bytes(read_bytes(data, &mut position, 4).try_into().unwrap());

            let script_sig_length = read_bytes(data, &mut position, 1)[0] as usize;
            let script_sig = read_bytes(data, &mut position, script_sig_length);

            let sequence =
                u32::from_le_bytes(read_bytes(data, &mut position, 4).try_into().unwrap());

            inputs.push(TxInput {
                previous_output: OutPoint {
                    txid: previous_txid,
                    vout: previous_vout,
                },
                script_sig,
                sequence,
            });
        }

        let output_count =
            u32::from_le_bytes(read_bytes(data, &mut position, 4).try_into().unwrap());
        let mut outputs = Vec::with_capacity(output_count as usize);

        for _ in 0..output_count {
            let value_in_satoshis =
                u64::from_le_bytes(read_bytes(data, &mut position, 8).try_into().unwrap());
            let script_pubkey_length = read_bytes(data, &mut position, 1)[0] as usize;
            let script_pubkey = read_bytes(data, &mut position, script_pubkey_length);
            outputs.push(TxOutput {
                value: value_in_satoshis,
                script_pubkey,
            });
        }

        let lock_time = u32::from_le_bytes(read_bytes(data, &mut position, 4).try_into().unwrap());

        Ok(LegacyTransaction {
            version,
            inputs,
            outputs,
            lock_time,
        })
    }
}

// Custom serialization for transaction
impl BitcoinSerialize for LegacyTransaction {
    fn serialize(&self) -> Vec<u8> {
        // TODO: Serialize only version and lock_time (simplified)
        let mut bytes = Vec::new();
        bytes.extend(&self.version.to_le_bytes());
        bytes.extend(&self.lock_time.to_le_bytes());
        bytes
    }
}
