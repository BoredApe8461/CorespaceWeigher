#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RelayChain {
    Polkadot,
    Kusama,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Parachain {
    /// Name of the parachain.
    pub name: String,
    /// The rpc url endpoint from where we can query the weight consumption.
    //
    // TODO: instead of having only one rpc url specified there should be a fallback.
    pub rpc_url: String,
    /// The `ParaId` of the parachain.
    pub para_id: u32,
    /// The relay chain that the parachain is using for block validation.
    pub relay_chain: RelayChain,
}

#[derive(Debug)]
pub struct WeightConsumption {
    /// The percentage of the weight used by user submitted extrinsics compared to the
    /// maximum potential.
    pub normal: f32,
    /// The percentage of the weight used by user operational dispatches compared to the
    /// maximum potential.
    pub operational: f32,
    /// The percentage of the weight used by the mandatory tasks of a parachain compared
    /// to the maximum potential.
    pub mandatory: f32,
}

impl std::fmt::Display for WeightConsumption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n\tNormal consumption: {}", self.normal)?;
        write!(f, "\n\tOperational consumption: {}", self.operational)?;
        write!(f, "\n\tMandatory consumption: {}", self.mandatory)?;
        Ok(())
    }
}
