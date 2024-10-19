TODO: Asmov Common Dataset
================================================================================

### unbreak tempo
make tempo work again now that we're moved and compiling

### an alternative to DatasetMut::take()
take is not atomic and could lead to race conditions.    
maybe a FnOnce() on the normal Dataset trait?

### sql db integration tests
sqlx's features may help with this.

### automatic ID creation
MemoryDataset::put() should create a local ID if an ID isn't provided