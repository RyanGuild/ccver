 Commit graph construction and traversal module.

 This module builds a directed graph representation of git commit history with
 memoized operations for efficient lookups and traversal.

 # Performance Characteristics

 - **Graph construction:** O(n) where n is the number of commits, ~800K elements/s throughput
 - **Parent/child lookups:** O(1) constant time via memoized adjacency lists
 - **Commit hash lookups:** O(1) constant time via hash map (~57ns)
 - **Head/tail access:** O(1) constant time via memoization
 - **Graph traversal (DFS/BFS):** O(n + e) where e is the number of edges

 The graph uses a layered architecture with composable memoization wrappers:
 1. `CommitMemo` - Memoizes commit hash lookups
 2. `WithParentsAndChildEdges` - Builds parent/child relationships
 3. `TailMemo` - Memoizes tail (root) commit
 4. `HeadMemo` - Memoizes head (current) commit
 5. `BranchMemo` - Memoizes branch information
 6. `WithCCVerVersions` - Assigns versions to all commits

 # Memory Usage

 - Node storage: `Arc<Mutex<CommitGraphNodeData>>` for thread-safe access
 - Hash map for O(1) commit lookups
 - Adjacency lists for O(1) parent/child lookups

 # Optimization Opportunities

 - Consider `RwLock` instead of `Mutex` for read-heavy workloads
 - Profile lock contention for large repositories
