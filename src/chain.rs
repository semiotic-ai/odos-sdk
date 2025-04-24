use alloy_chains::NamedChain;
use alloy_primitives::Address;

use crate::{
    ODOS_V2_ARBITRUM_ROUTER, ODOS_V2_AVALANCHE_ROUTER, ODOS_V2_BASE_ROUTER, ODOS_V2_BSC_ROUTER,
    ODOS_V2_ETHEREUM_ROUTER, ODOS_V2_FANTOM_ROUTER, ODOS_V2_FRAXTAL_ROUTER, ODOS_V2_LINEA_ROUTER,
    ODOS_V2_MANTLE_ROUTER, ODOS_V2_MODE_ROUTER, ODOS_V2_OP_ROUTER, ODOS_V2_POLYGON_ROUTER,
    ODOS_V2_SCROLL_ROUTER, ODOS_V2_SONIC_ROUTER, ODOS_V2_ZKSYNC_ROUTER,
};

pub trait OdosChain {
    fn v2_router_address(&self) -> Address;
}

impl OdosChain for NamedChain {
    fn v2_router_address(&self) -> Address {
        use NamedChain::*;
        match self {
            Arbitrum => ODOS_V2_ARBITRUM_ROUTER,
            Avalanche => ODOS_V2_AVALANCHE_ROUTER,
            Base => ODOS_V2_BASE_ROUTER,
            BinanceSmartChain => ODOS_V2_BSC_ROUTER,
            Fantom => ODOS_V2_FANTOM_ROUTER,
            Fraxtal => ODOS_V2_FRAXTAL_ROUTER,
            Mainnet => ODOS_V2_ETHEREUM_ROUTER,
            Optimism => ODOS_V2_OP_ROUTER,
            Polygon => ODOS_V2_POLYGON_ROUTER,
            Linea => ODOS_V2_LINEA_ROUTER,
            Mantle => ODOS_V2_MANTLE_ROUTER,
            Mode => ODOS_V2_MODE_ROUTER,
            Scroll => ODOS_V2_SCROLL_ROUTER,
            Sonic => ODOS_V2_SONIC_ROUTER,
            ZkSync => ODOS_V2_ZKSYNC_ROUTER,
            _ => unimplemented!(),
        }
        .parse()
        .unwrap()
    }
}
