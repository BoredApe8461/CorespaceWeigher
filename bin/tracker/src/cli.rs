use clap::Parser;

/// Arguments for the tracker.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Specifies the index of the RPC to be used.
	///
	/// Multiple RPCs may be provided for each parachain on Kusama and Polkadot.
	/// `rpc_index` selects which RPC from the list will be used.
	#[arg(short, long)]
	rpc_index: u8,
}
