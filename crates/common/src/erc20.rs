use alloy::primitives::U256;
use alloy::sol_types::SolCall;
use alloy::{hex::FromHex, primitives::Address};
use alloy_ethers_typecast::transaction::{
    ReadContractParameters, ReadableClientError, ReadableClientHttp,
};
use alloy_ethers_typecast::{
    multicall::{
        IMulticall3::{aggregate3Call, Call3},
        MULTICALL3_ADDRESS,
    },
    transaction::ReadContractParametersBuilderError,
};
use rain_error_decoding::{AbiDecodeFailedErrors, AbiDecodedErrorType};
use rain_orderbook_bindings::IERC20::{balanceOfCall, decimalsCall, nameCall, symbolCall};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use url::Url;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TokenInfo {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TokenInfo);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ERC20 {
    pub rpcs: Vec<Url>,
    pub address: Address,
}
impl ERC20 {
    pub fn new(rpcs: Vec<Url>, address: Address) -> Self {
        Self { rpcs, address }
    }

    async fn get_client(&self) -> Result<ReadableClientHttp, Error> {
        ReadableClientHttp::new_from_urls(self.rpcs.iter().map(|rpc| rpc.to_string()).collect())
            .map_err(|err| Error::ReadableClientError {
                msg: format!("rpcs: {:?}", self.rpcs),
                source: err,
            })
    }

    pub async fn decimals(&self) -> Result<u8, Error> {
        let client = self.get_client().await?;
        let parameters = ReadContractParameters {
            address: self.address,
            call: decimalsCall {},
            block_number: None,
            gas: None,
        };
        Ok(client
            .read(parameters)
            .await
            .map_err(|err| Error::ReadableClientError {
                msg: format!("address: {}", self.address),
                source: err,
            })?
            ._0)
    }

    pub async fn name(&self) -> Result<String, Error> {
        let client = self.get_client().await?;
        let parameters = ReadContractParameters {
            address: self.address,
            call: nameCall {},
            block_number: None,
            gas: None,
        };
        Ok(client
            .read(parameters)
            .await
            .map_err(|err| Error::ReadableClientError {
                msg: format!("address: {}", self.address),
                source: err,
            })?
            ._0)
    }

    pub async fn symbol(&self) -> Result<String, Error> {
        let client = self.get_client().await?;
        let parameters = ReadContractParameters {
            address: self.address,
            call: symbolCall {},
            block_number: None,
            gas: None,
        };
        Ok(client
            .read(parameters)
            .await
            .map_err(|err| Error::ReadableClientError {
                msg: format!("address: {}", self.address),
                source: err,
            })?
            ._0)
    }

    pub async fn token_info(&self, multicall_address: Option<String>) -> Result<TokenInfo, Error> {
        let client = self.get_client().await?;

        let results = client
            .read(ReadContractParameters {
                gas: None,
                address: multicall_address
                    .map_or(Address::from_hex(MULTICALL3_ADDRESS).unwrap(), |s| {
                        Address::from_str(&s).unwrap_or(Address::default())
                    }),
                call: aggregate3Call {
                    calls: vec![
                        Call3 {
                            target: self.address,
                            allowFailure: false,
                            callData: decimalsCall {}.abi_encode().into(),
                        },
                        Call3 {
                            target: self.address,
                            allowFailure: false,
                            callData: nameCall {}.abi_encode().into(),
                        },
                        Call3 {
                            target: self.address,
                            allowFailure: false,
                            callData: symbolCall {}.abi_encode().into(),
                        },
                    ],
                },
                block_number: None,
            })
            .await
            .map_err(|err| Error::ReadableClientError {
                msg: format!("address: {}", self.address),
                source: err,
            })?;

        Ok(TokenInfo {
            decimals: decimalsCall::abi_decode_returns(&results.returnData[0].returnData, true)
                .map_err(|err| Error::SolTypesError {
                    msg: format!("address: {}", self.address),
                    source: err,
                })?
                ._0,
            name: nameCall::abi_decode_returns(&results.returnData[1].returnData, true)
                .map_err(|err| Error::SolTypesError {
                    msg: format!("address: {}", self.address),
                    source: err,
                })?
                ._0,
            symbol: symbolCall::abi_decode_returns(&results.returnData[2].returnData, true)
                .map_err(|err| Error::SolTypesError {
                    msg: format!("address: {}", self.address),
                    source: err,
                })?
                ._0,
        })
    }

    pub async fn get_account_balance(&self, account: Address) -> Result<U256, Error> {
        let client = self.get_client().await?;
        let parameters = ReadContractParameters {
            address: self.address,
            call: balanceOfCall { account },
            block_number: None,
            gas: None,
        };
        Ok(client
            .read(parameters)
            .await
            .map_err(|err| Error::ReadableClientError {
                msg: format!("address: {}", self.address),
                source: err,
            })?
            ._0)
    }
}

const ERROR_MESSAGE: &str = "Failed to get token information: ";

#[derive(Debug, Error)]
pub enum Error {
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    ReadContractError {
        msg: String,
        #[source]
        source: ReadContractParametersBuilderError,
    },
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    ReadableClientError {
        msg: String,
        #[source]
        source: ReadableClientError,
    },
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    AbiDecodedErrorType {
        msg: String,
        #[source]
        source: AbiDecodedErrorType,
    },
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    AbiDecodeError {
        msg: String,
        #[source]
        source: AbiDecodeFailedErrors,
    },
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    SolTypesError {
        msg: String,
        #[source]
        source: alloy::sol_types::Error,
    },
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use alloy::{hex, sol_types::SolValue};
    use alloy_ethers_typecast::rpc::Response;
    use httpmock::MockServer;

    #[tokio::test]
    async fn test_decimals() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x313ce567");
            then.body(
                Response::new_success(
                    1,
                    "0x0000000000000000000000000000000000000000000000000000000000000012",
                )
                .to_json_string()
                .unwrap(),
            );
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );

        let decimals = erc20.decimals().await.unwrap();
        assert_eq!(decimals, 18);
    }

    #[tokio::test]
    async fn test_decimals_invalid() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x313ce567");
            then.body(Response::new_success(1, "0x1").to_json_string().unwrap());
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        assert!(erc20.decimals().await.is_err());

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x313ce567");
            then.body(
                Response::new_success(
                    1,
                    "0x0000000000000000000000000000000000000000000000000000000000000123",
                )
                .to_json_string()
                .unwrap(),
            );
        });
        assert!(erc20.decimals().await.is_err());
    }

    #[tokio::test]
    async fn test_name() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x06fdde03");
            then.body(
                Response::new_success(
                    1,
                    &hex::encode_prefixed("Test Token".to_string().abi_encode()).to_string(),
                )
                .to_json_string()
                .unwrap(),
            );
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        let name = erc20.name().await.unwrap();
        assert_eq!(name, "Test Token");
    }

    #[tokio::test]
    async fn test_name_invalid() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x06fdde03");
            then.body(Response::new_success(1, "0x1").to_json_string().unwrap());
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        assert!(erc20.name().await.is_err());

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x06fdde03");
            then.body(
                Response::new_success(
                    1,
                    "0x0000000000000000000000000000000000000000000000000000000000000123",
                )
                .to_json_string()
                .unwrap(),
            );
        });
        assert!(erc20.name().await.is_err());
    }

    #[tokio::test]
    async fn test_symbol() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x95d89b41");
            then.body(
                Response::new_success(
                    1,
                    &hex::encode_prefixed("TEST".to_string().abi_encode()).to_string(),
                )
                .to_json_string()
                .unwrap(),
            );
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        let symbol = erc20.symbol().await.unwrap();
        assert_eq!(symbol, "TEST");
    }

    #[tokio::test]
    async fn test_symbol_invalid() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x95d89b41");
            then.body(Response::new_success(1, "0x1").to_json_string().unwrap());
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        assert!(erc20.symbol().await.is_err());

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x95d89b41");
            then.body(
                Response::new_success(
                    1,
                    "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000205445535400000000000000000000000000000000000000000000000000000000",
                )
                .to_json_string()
                .unwrap(),
            );
        });
        assert!(erc20.symbol().await.is_err());
    }

    #[tokio::test]
    async fn test_token_info() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x82ad56cb");
            then.body(Response::new_success(
                1,
                "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000",
            )
            .to_json_string()
            .unwrap(),
            );
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        let token_info = erc20.token_info(None).await.unwrap();

        assert_eq!(token_info.decimals, 6);
        assert_eq!(token_info.name, "Token 1");
        assert_eq!(token_info.symbol, "T1");
    }

    #[tokio::test]
    async fn test_token_info_invalid() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x82ad56cb");
            then.body(Response::new_success(1, "0x1").to_json_string().unwrap());
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        assert!(erc20.token_info(None).await.is_err());

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x82ad56cb");
            then.body(
                Response::new_success(1, "0x00000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000012300000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000").to_json_string().unwrap(),
            );
        });
        assert!(erc20.token_info(None).await.is_err());
    }

    #[tokio::test]
    async fn test_get_account_balance() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x70a08231");
            then.body(
                Response::new_success(
                    1,
                    "0x00000000000000000000000000000000000000000000000000000000000003e8",
                )
                .to_json_string()
                .unwrap(),
            );
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );

        let balance = erc20.get_account_balance(Address::ZERO).await.unwrap();
        assert_eq!(balance, alloy::primitives::U256::from(1000u64));
    }
}
