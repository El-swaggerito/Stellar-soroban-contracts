# Enterprise-Grade History APIs

## Overview

This document describes the enterprise-grade read APIs for accessing on-chain historical data. These APIs provide auditors, compliance officers, and dashboard builders with reliable, paginated access to proposal, voting, execution, and slashing history.

## API Design Principles

1. **Pagination Support**: All history APIs support pagination to handle large datasets efficiently
2. **Consistent Sorting**: Configurable ascending/descending order for chronological or reverse-chronological queries
3. **Type Safety**: Strongly-typed response structures with metadata
4. **Gas Efficiency**: Bounded queries with configurable limits to prevent excessive gas consumption
5. **Audit Trail**: Complete historical records for compliance and transparency

---

## Data Structures

### Pagination Parameters

```rust
pub struct PaginationParams {
    pub offset: u32,        // Starting position (0-indexed)
    pub limit: u32,         // Number of items to return (max 100)
    pub sort_ascending: bool, // true = oldest first, false = newest first
}
```

### Pagination Metadata

```rust
pub struct PaginationInfo {
    pub total_count: u32,     // Total items available
    pub returned_count: u32,  // Items returned in this response
    pub offset: u32,          // Current offset used
    pub limit: u32,           // Current limit used
    pub has_more: bool,       // Whether more items exist after this page
}
```

### Proposal History Entry

```rust
pub struct ProposalHistoryEntry {
    pub proposal_id: u64,
    pub token_id: TokenId,
    pub description_hash: Hash,
    pub quorum: u128,
    pub for_votes: u128,
    pub against_votes: u128,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub executed_at: Option<u64>,
    pub creator: AccountId,
}
```

### Vote History Entry

```rust
pub struct VoteHistoryEntry {
    pub proposal_id: u64,
    pub token_id: TokenId,
    pub voter: AccountId,
    pub support: bool,
    pub vote_weight: u128,
    pub voted_at: u64,
}
```

### Execution History Entry

```rust
pub struct ExecutionHistoryEntry {
    pub proposal_id: u64,
    pub token_id: TokenId,
    pub executed_at: u64,
    pub passed: bool,
    pub executor: AccountId,
    pub transaction_hash: Hash,
}
```

### Slashing History Entry

```rust
pub struct SlashingHistoryEntry {
    pub target: AccountId,
    pub role: SlashingRole,
    pub reason: SlashingReason,
    pub slashed_amount: u128,
    pub slashed_at: u64,
    pub transaction_hash: Hash,
    pub authority: AccountId,
    pub repeat_offense_count: u32,
}
```

### Slashing Role Enum

```rust
pub enum SlashingRole {
    OracleProvider,
    GovernanceParticipant,
    RiskPoolProvider,
    ClaimSubmitter,
    BridgeOperator,
    Other(String),
}
```

### Slashing Reason Enum

```rust
pub enum SlashingReason {
    OracleManipulation,
    GovernanceAttack,
    DoubleSigning,
    ComplianceViolation,
    MaliciousBehavior,
    Negligence,
    Custom(String),
}
```

---

## API Endpoints

### 1. get_proposal_history

Retrieves the complete history of proposals for a specific token.

**Signature:**
```rust
pub fn get_proposal_history(
    &self,
    token_id: TokenId,
    params: PaginationParams,
) -> PaginatedProposalHistory
```

**Parameters:**
- `token_id`: The token ID to query proposals for
- `params`: Pagination parameters

**Response:**
```rust
pub struct PaginatedProposalHistory {
    pub entries: Vec<ProposalHistoryEntry>,
    pub pagination: PaginationInfo,
}
```

**Example Usage:**
```rust
// Get first 20 proposals (oldest first)
let params = PaginationParams {
    offset: 0,
    limit: 20,
    sort_ascending: true,
};
let history = contract.get_proposal_history(token_id, params);
```

---

### 2. get_vote_history

Retrieves all votes cast on a specific proposal.

**Signature:**
```rust
pub fn get_vote_history(
    &self,
    token_id: TokenId,
    proposal_id: u64,
    params: PaginationParams,
) -> PaginatedVoteHistory
```

**Parameters:**
- `token_id`: The token ID
- `proposal_id`: The proposal ID to query votes for
- `params`: Pagination parameters

**Response:**
```rust
pub struct PaginatedVoteHistory {
    pub entries: Vec<VoteHistoryEntry>,
    pub pagination: PaginationInfo,
}
```

**Example Usage:**
```rust
// Get last 50 votes (newest first)
let params = PaginationParams {
    offset: 0,
    limit: 50,
    sort_ascending: false,
};
let votes = contract.get_vote_history(token_id, proposal_id, params);
```

---

### 3. get_execution_history

Retrieves the history of all proposal executions across the system.

**Signature:**
```rust
pub fn get_execution_history(
    &self,
    params: PaginationParams,
) -> PaginatedExecutionHistory
```

**Parameters:**
- `params`: Pagination parameters

**Response:**
```rust
pub struct PaginatedExecutionHistory {
    pub entries: Vec<ExecutionHistoryEntry>,
    pub pagination: PaginationInfo,
}
```

**Example Usage:**
```rust
// Get recent executions (newest first)
let params = PaginationParams {
    offset: 0,
    limit: 30,
    sort_ascending: false,
};
let executions = contract.get_execution_history(params);
```

---

### 4. record_slashing

Records a slashing event (admin/governance only).

**Signature:**
```rust
pub fn record_slashing(
    &mut self,
    target: AccountId,
    role: SlashingRole,
    reason: SlashingReason,
    slashed_amount: u128,
    authority: AccountId,
) -> Result<(), Error>
```

**Parameters:**
- `target`: The account being slashed
- `role`: The role of the slashed account
- `reason`: The reason for slashing
- `slashed_amount`: Amount of funds slashed
- `authority`: The authority executing the slash

**Response:**
- `Ok(())` on success
- `Error::Unauthorized` if caller is not authorized

**Example Usage:**
```rust
contract.record_slashing(
    oracle_account,
    SlashingRole::OracleProvider,
    SlashingReason::OracleManipulation,
    1000000, // slashed amount
    governance_contract,
)?;
```

---

### 5. get_slashing_history

Retrieves slashing history with optional filtering.

**Signature:**
```rust
pub fn get_slashing_history(
    &self,
    target: Option<AccountId>,
    role: Option<SlashingRole>,
    params: PaginationParams,
) -> PaginatedSlashingHistory
```

**Parameters:**
- `target`: Optional filter by target account
- `role`: Optional filter by role
- `params`: Pagination parameters

**Response:**
```rust
pub struct PaginatedSlashingHistory {
    pub entries: Vec<SlashingHistoryEntry>,
    pub pagination: PaginationInfo,
}
```

**Example Usage:**
```rust
// Get all slashing events for a specific role
let params = PaginationParams {
    offset: 0,
    limit: 50,
    sort_ascending: true,
};
let history = contract.get_slashing_history(
    None, // no target filter
    Some(SlashingRole::OracleProvider),
    params,
);
```

---

## JSON Schema

### Proposal History Response Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ProposalHistoryResponse",
  "type": "object",
  "properties": {
    "entries": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "proposal_id": { "type": "string" },
          "token_id": { "type": "string" },
          "description_hash": { "type": "string" },
          "quorum": { "type": "string" },
          "for_votes": { "type": "string" },
          "against_votes": { "type": "string" },
          "status": { 
            "type": "string",
            "enum": ["Open", "Executed", "Rejected", "Closed"]
          },
          "created_at": { "type": "string", "format": "date-time" },
          "executed_at": { "type": "string", "format": "date-time", "nullable": true },
          "creator": { "type": "string" }
        }
      }
    },
    "pagination": {
      "type": "object",
      "properties": {
        "total_count": { "type": "integer" },
        "returned_count": { "type": "integer" },
        "offset": { "type": "integer" },
        "limit": { "type": "integer" },
        "has_more": { "type": "boolean" }
      }
    }
  }
}
```

### Vote History Response Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "VoteHistoryResponse",
  "type": "object",
  "properties": {
    "entries": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "proposal_id": { "type": "string" },
          "token_id": { "type": "string" },
          "voter": { "type": "string" },
          "support": { "type": "boolean" },
          "vote_weight": { "type": "string" },
          "voted_at": { "type": "string", "format": "date-time" }
        }
      }
    },
    "pagination": {
      "type": "object",
      "properties": {
        "total_count": { "type": "integer" },
        "returned_count": { "type": "integer" },
        "offset": { "type": "integer" },
        "limit": { "type": "integer" },
        "has_more": { "type": "boolean" }
      }
    }
  }
}
```

### Execution History Response Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ExecutionHistoryResponse",
  "type": "object",
  "properties": {
    "entries": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "proposal_id": { "type": "string" },
          "token_id": { "type": "string" },
          "executed_at": { "type": "string", "format": "date-time" },
          "passed": { "type": "boolean" },
          "executor": { "type": "string" },
          "transaction_hash": { "type": "string" }
        }
      }
    },
    "pagination": {
      "type": "object",
      "properties": {
        "total_count": { "type": "integer" },
        "returned_count": { "type": "integer" },
        "offset": { "type": "integer" },
        "limit": { "type": "integer" },
        "has_more": { "type": "boolean" }
      }
    }
  }
}
```

### Slashing History Response Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SlashingHistoryResponse",
  "type": "object",
  "properties": {
    "entries": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "target": { "type": "string" },
          "role": { 
            "type": "string",
            "enum": [
              "OracleProvider",
              "GovernanceParticipant",
              "RiskPoolProvider",
              "ClaimSubmitter",
              "BridgeOperator",
              "Other"
            ]
          },
          "reason": {
            "type": "string",
            "enum": [
              "OracleManipulation",
              "GovernanceAttack",
              "DoubleSigning",
              "ComplianceViolation",
              "MaliciousBehavior",
              "Negligence",
              "Custom"
            ]
          },
          "slashed_amount": { "type": "string" },
          "slashed_at": { "type": "string", "format": "date-time" },
          "transaction_hash": { "type": "string" },
          "authority": { "type": "string" },
          "repeat_offense_count": { "type": "integer" }
        }
      }
    },
    "pagination": {
      "type": "object",
      "properties": {
        "total_count": { "type": "integer" },
        "returned_count": { "type": "integer" },
        "offset": { "type": "integer" },
        "limit": { "type": "integer" },
        "has_more": { "type": "boolean" }
      }
    }
  }
}
```

---

## Frontend Query Patterns

### React/TypeScript Example

```typescript
interface PaginationParams {
  offset: number;
  limit: number;
  sort_ascending: boolean;
}

interface ProposalHistoryEntry {
  proposal_id: string;
  token_id: string;
  description_hash: string;
  quorum: string;
  for_votes: string;
  against_votes: string;
  status: 'Open' | 'Executed' | 'Rejected' | 'Closed';
  created_at: string;
  executed_at: string | null;
  creator: string;
}

interface PaginatedProposalHistory {
  entries: ProposalHistoryEntry[];
  pagination: {
    total_count: number;
    returned_count: number;
    offset: number;
    limit: number;
    has_more: boolean;
  };
}

async function fetchProposalHistory(
  contract: any,
  tokenId: string,
  page: number = 0,
  pageSize: number = 20
): Promise<PaginatedProposalHistory> {
  const params = {
    offset: page * pageSize,
    limit: pageSize,
    sort_ascending: false, // newest first
  };
  
  const result = await contract.query.get_proposal_history(tokenId, params);
  return result;
}

// Usage in a React component
function ProposalHistory({ tokenId }: { tokenId: string }) {
  const [history, setHistory] = useState<ProposalHistoryEntry[]>([]);
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    async function load() {
      setLoading(true);
      const result = await fetchProposalHistory(contract, tokenId, page);
      setHistory(result.entries);
      setLoading(false);
    }
    load();
  }, [tokenId, page]);

  return (
    <div>
      {loading ? (
        <div>Loading...</div>
      ) : (
        <>
          <table>
            <thead>
              <tr>
                <th>Proposal ID</th>
                <th>Status</th>
                <th>For Votes</th>
                <th>Against Votes</th>
                <th>Created</th>
              </tr>
            </thead>
            <tbody>
              {history.map(entry => (
                <tr key={entry.proposal_id}>
                  <td>{entry.proposal_id}</td>
                  <td>{entry.status}</td>
                  <td>{entry.for_votes}</td>
                  <td>{entry.against_votes}</td>
                  <td>{new Date(entry.created_at).toLocaleString()}</td>
                </tr>
              ))}
            </tbody>
          </table>
          
          <div className="pagination">
            <button 
              onClick={() => setPage(p => Math.max(0, p - 1))}
              disabled={page === 0}
            >
              Previous
            </button>
            <span>Page {page + 1}</span>
            <button 
              onClick={() => setPage(p => p + 1)}
              disabled={!hasMore}
            >
              Next
            </button>
          </div>
        </>
      )}
    </div>
  );
}
```

### Vue.js Example

```vue
<template>
  <div class="vote-history">
    <div v-if="loading">Loading votes...</div>
    <div v-else>
      <table>
        <thead>
          <tr>
            <th>Voter</th>
            <th>Support</th>
            <th>Weight</th>
            <th>Timestamp</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="vote in votes" :key="vote.voter">
            <td>{{ formatAddress(vote.voter) }}</td>
            <td :class="vote.support ? 'yes' : 'no'">
              {{ vote.support ? 'Yes' : 'No' }}
            </td>
            <td>{{ formatWeight(vote.vote_weight) }}</td>
            <td>{{ formatDate(vote.voted_at) }}</td>
          </tr>
        </tbody>
      </table>
      
      <div class="pagination-info">
        Showing {{ pagination.returned_count }} of {{ pagination.total_count }} votes
        <button @click="loadNextPage" :disabled="!pagination.has_more">
          Load More
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

const props = defineProps<{
  tokenId: string;
  proposalId: number;
}>();

const votes = ref<VoteHistoryEntry[]>([]);
const pagination = ref<PaginationInfo>({ 
  total_count: 0, 
  returned_count: 0, 
  offset: 0, 
  limit: 20, 
  has_more: false 
});
const loading = ref(false);
const currentPage = ref(0);

async function loadVotes(page: number = 0) {
  loading.value = true;
  try {
    const params = {
      offset: page * 20,
      limit: 20,
      sort_ascending: false
    };
    
    const result = await contract.query.get_vote_history(
      props.tokenId,
      props.proposalId,
      params
    );
    
    votes.value = result.entries;
    pagination.value = result.pagination;
    currentPage.value = page;
  } finally {
    loading.value = false;
  }
}

function loadNextPage() {
  if (pagination.value.has_more) {
    loadVotes(currentPage.value + 1);
  }
}

function formatAddress(addr: string): string {
  return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
}

function formatWeight(weight: string): string {
  return (parseInt(weight) / 1e12).toFixed(2);
}

function formatDate(timestamp: string): string {
  return new Date(timestamp).toLocaleString();
}

// Initial load
loadVotes();
</script>
```

---

## Integration Guide

### Step 1: Contract Setup

Ensure your contract includes the history tracking mappings and structures defined in this document.

### Step 2: Indexing Strategy

For optimal performance with large datasets:
- Use the built-in pagination to limit query sizes
- Cache frequently accessed history locally
- Consider off-chain indexing for complex queries

### Step 3: Error Handling

```typescript
async function safeQuery<T>(
  queryFn: () => Promise<T>,
  retries: number = 3
): Promise<T> {
  for (let i = 0; i < retries; i++) {
    try {
      return await queryFn();
    } catch (error) {
      if (i === retries - 1) throw error;
      await sleep(1000 * (i + 1)); // exponential backoff
    }
  }
  throw new Error('Unexpected error');
}
```

### Step 4: Monitoring

Track these metrics for history API usage:
- Average query response time
- Pagination depth (how many pages users typically access)
- Most queried tokens/proposals
- Rate limiting needs

---

## Best Practices

### 1. Pagination Limits
- Always use reasonable limits (recommended: 20-100 items per page)
- Display pagination metadata to users
- Provide "Load More" or infinite scroll UX

### 2. Sorting
- Default to descending order (newest first) for most use cases
- Allow users to toggle sort direction
- Maintain consistent ordering within pages

### 3. Caching
- Cache static history (old proposals/executions)
- Invalidate cache on new events
- Use ETags or similar mechanisms for HTTP caching

### 4. Performance
- Avoid querying entire history
- Use filters when available (especially for slashing history)
- Batch multiple queries when possible

### 5. Audit Compliance
- Log all history queries for audit trails
- Preserve historical data integrity
- Implement proper access controls

---

## Gas Optimization

### Query Costs

The gas cost for history queries depends on:
- Number of items retrieved (limit parameter)
- Size of each entry
- Storage retrieval complexity

### Optimization Tips

```rust
// ✅ Good: Bounded query with reasonable limit
let params = PaginationParams {
    offset: 0,
    limit: 50,
    sort_ascending: false,
};

// ❌ Bad: Unbounded query (could exhaust gas)
let params = PaginationParams {
    offset: 0,
    limit: 10000,
    sort_ascending: true,
};
```

---

## Testing

### Unit Test Example

```rust
#[test]
fn test_proposal_history_pagination() {
    let mut contract = PropertyToken::new();
    
    // Create multiple proposals
    for i in 1..=50 {
        contract.create_proposal(
            token_id,
            1000,
            Hash::from([i; 32])
        ).unwrap();
    }
    
    // Test pagination
    let params = PaginationParams {
        offset: 0,
        limit: 20,
        sort_ascending: true,
    };
    
    let result = contract.get_proposal_history(token_id, params);
    
    assert_eq!(result.pagination.total_count, 50);
    assert_eq!(result.pagination.returned_count, 20);
    assert_eq!(result.entries.len(), 20);
    assert!(result.pagination.has_more);
    
    // Test second page
    let params2 = PaginationParams {
        offset: 20,
        limit: 20,
        sort_ascending: true,
    };
    
    let result2 = contract.get_proposal_history(token_id, params2);
    assert_eq!(result2.pagination.offset, 20);
    assert_ne!(result2.entries[0].proposal_id, result.entries[0].proposal_id);
}
```

---

## Troubleshooting

### Common Issues

**Issue: Empty results despite expecting data**
- Check that `token_id` and `proposal_id` are correct
- Verify the offset isn't beyond total count
- Ensure data has been written to history (check events)

**Issue: Slow query performance**
- Reduce the limit parameter
- Use filters (for slashing history)
- Consider caching strategies

**Issue: Inconsistent ordering**
- Verify `sort_ascending` parameter
- Check that history was recorded in correct order
- Ensure no concurrent modifications during query

---

## Security Considerations

1. **Access Control**: All history APIs are read-only and publicly accessible
2. **Data Integrity**: History entries are immutable once recorded
3. **Privacy**: No sensitive data is exposed through history APIs
4. **Rate Limiting**: Implement client-side rate limiting to prevent abuse

---

## Version History

- **v1.0** (2026-03-26): Initial implementation
  - `get_proposal_history` with pagination
  - `get_vote_history` with pagination
  - `get_execution_history` with pagination
  - `get_slashing_history` with filtering and pagination
  - `record_slashing` for governance-controlled slashing

---

## Related Documentation

- [Architecture](./architecture.md): System overview and design
- [Contracts](./contracts.md): Complete contract API reference
- [DAO Risk Architecture](./dao-risk-architecture.md): Risk metrics and monitoring
- [Integration Guide](./integration.md): General integration patterns
