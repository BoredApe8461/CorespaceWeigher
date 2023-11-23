pub struct Parachain {
    /// Name of the parachain.
    pub name: String,
    /// The rpc url endpoint from where we can query the weight consumption.
    //
    // TODO: instead of having only one rpc ulr specified there should be a fallback.
    pub rpc_url: String,
    /// The `ParaId` of the parachain. 
    pub para_id: u32,
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
