use ethers::prelude::*;

const WSS_URL: &str = "wss://localhost:8546";

/// Ideas:
/// - look through all the major builders and understand their strategies.
/// - look through new transactions coming in and check contract creations.
/// - scan through MEV builders / relay APIs for information
/// - put things into a sqlite that is easy to query
/// - perhaps transform into graphql?
/// - how can we detect if searchers built their stuff from private
///   order flows compared to using public mempools?
#[tokio::main]
async fn main() -> eyre::Result<()> {
    // A Ws provider can be created from a ws(s) URI.
    // In case of wss you must add the "rustls" or "openssl" feature
    // to the ethers library dependency in `Cargo.toml`.
    let provider = Provider::<Ws>::connect(WSS_URL).await?;

    let mut stream = provider.subscribe_blocks().await?.take(1);
    while let Some(block) = stream.next().await {
        println!("{:?}", block.hash);
    }

    Ok(())
}
