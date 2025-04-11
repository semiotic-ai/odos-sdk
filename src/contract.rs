use alloy_chains::NamedChain;
use alloy_primitives::Address;

/// <https://arbiscan.io/address/0xa669e7a0d4b3e4fa48af2de86bd4cd7126be4e13>
pub const ODOS_V2_ARBITRUM_ROUTER: &str = "0xa669e7a0d4b3e4fa48af2de86bd4cd7126be4e13";

/// <https://basescan.org/address/0x19cEeAd7105607Cd444F5ad10dd51356436095a1>
pub const ODOS_V2_BASE_ROUTER: &str = "0x19cEeAd7105607Cd444F5ad10dd51356436095a1";

/// <https://bscscan.com/address/0x89b8aa89fdd0507a99d334cbe3c808fafc7d850e>
pub const ODOS_V2_BSC_ROUTER: &str = "0x89b8AA89FDd0507a99d334CBe3C808fAFC7d850E";

/// <https://etherscan.io/address/0xcf5540fffcdc3d510b18bfca6d2b9987b0772559>
pub const ODOS_V2_ETHEREUM_ROUTER: &str = "0xCf5540fFFCdC3d510B18bFcA6d2b9987b0772559";

/// <https://optimistic.etherscan.io/address/0xca423977156bb05b13a2ba3b76bc5419e2fe9680>
pub const ODOS_V2_OP_ROUTER: &str = "0xCa423977156BB05b13A2BA3b76Bc5419E2fE9680";

/// <https://snowtrace.io/address/0x88de50b233052e4fb783d4f6db78cc34fea3e9fc>
pub const ODOS_V2_AVALANCHE_ROUTER: &str = "0x88de50B233052e4Fb783d4F6db78Cc34fEa3e9FC";

/// <https://polygonscan.com/address/0x4e3288c9ca110bcc82bf38f09a7b425c095d92bf>
pub const ODOS_V2_POLYGON_ROUTER: &str = "0x4e3288c9ca110bcc82bf38f09a7b425c095d92bf";

/// <https://ftmscan.com/address/0xD0c22A5435F4E8E5770C1fAFb5374015FC12F7cD>
pub const ODOS_V2_FANTOM_ROUTER: &str = "0xD0c22A5435F4E8E5770C1fAFb5374015FC12F7cD";

/// <https://linea.blockscout.com/address/0x2d8879046f1559E53eb052E949e9544bCB72f414>
pub const ODOS_V2_LINEA_ROUTER: &str = "0x2d8879046f1559E53eb052E949e9544bCB72f414";

/// <https://explorer.mode.network/address/0x7E15EB462cdc67Cf92Af1f7102465a8F8c784874>
pub const ODOS_V2_MODE_ROUTER: &str = "0x7E15EB462cdc67Cf92Af1f7102465a8F8c784874";

/// <https://explorer.scroll.io/address/0xbFe03C9E20a9Fc0b37de01A172F207004935E0b1>
pub const ODOS_V2_SCROLL_ROUTER: &str = "0xbFe03C9E20a9Fc0b37de01A172F207004935E0b1";

pub trait OdosContract {
    fn v2_router_address(&self) -> Address;
}

impl OdosContract for NamedChain {
    fn v2_router_address(&self) -> Address {
        use NamedChain::*;
        match self {
            Arbitrum => ODOS_V2_ARBITRUM_ROUTER,
            Base => ODOS_V2_BASE_ROUTER,
            BinanceSmartChain => ODOS_V2_BSC_ROUTER,
            Mainnet => ODOS_V2_ETHEREUM_ROUTER,
            Optimism => ODOS_V2_OP_ROUTER,
            Avalanche => ODOS_V2_AVALANCHE_ROUTER,
            Polygon => ODOS_V2_POLYGON_ROUTER,
            Fantom => ODOS_V2_FANTOM_ROUTER,
            Linea => ODOS_V2_LINEA_ROUTER,
            Mode => ODOS_V2_MODE_ROUTER,
            Scroll => ODOS_V2_SCROLL_ROUTER,
            _ => unimplemented!(),
        }
        .parse()
        .unwrap()
    }
}
