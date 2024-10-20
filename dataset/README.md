Asmov Common Dataset
================================================================================
[![Latest Version]][crates.io]

[Latest Version]: https://img.shields.io/crates/v/asmov-common-dataset.svg
[crates.io]: https://crates.io/crates/asmov-common-dataset

Library for application data modeling between clients and servers.

The overall goal of this project is to allow as much code reuse as possible when dealing with both data modeling and data access between client applications and backend servers.

A **dataset** groups Rust data structs from a single data domain and provides a read/write API for some protocol (in-memory, sqlite, postgres, indexdb, strategic).

App and server code typically interact solely with the **strategic dataset**. This dataset  acts as an umbrella, maintaining all other local and remote datasets internally. As its name implies, it strategically selects the right internal dataset for the job, managing cache and propogating mutation automatically.

A client keeps a cached in-memory dataset of models that it is using, while maintaining a connection to another dataset that is acting as a single-source-of-truth (SSOT). A client SSOT is typically either a remote backend API or a local lightweight database API such as SQLite or IndexedDB. The SSOT dataset will invalidate cache live as necessary by propogating events back through the client's strategic dataset. The strategic dataset then propogates changes through its components. Conversely, when writes occur on the client, the strategic dataset passes them through the memory dataset first, and then the SSOT dataset.

On the other side, a backend server's strategy typically maintains an in-memory cache dataset and a PostgreSQL dataset. The PostgreSQL database server has extensions installed that allow the backend server's strategic dataset to invalidate cache properly and propogate those changes back to the backend server's clients.

Repository
--------------------------------------------------------------------------------
Contributors, please review [ASMOV.md](./ASMOV.md).  

Found a bug? Search for an existing issue on GitHub.  
If an issue exists, chime in to add weight to it.  
If an issue does not exist, create one and tell us how to reproduce the bug. 


License (AGPL3)
--------------------------------------------------------------------------------
Asmov Common Dataset: Library for application data modeling between clients and servers.  
Copyright (C) 2024 Asmov LLC  

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a [copy](./LICENSE.txt) of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

