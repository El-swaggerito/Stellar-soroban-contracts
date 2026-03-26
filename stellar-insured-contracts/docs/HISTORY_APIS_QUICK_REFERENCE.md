# History APIs Quick Reference

## At a Glance

### Available Endpoints

| Endpoint | Purpose | Pagination | Filters |
|----------|---------|------------|---------|
| `get_proposal_history` | Retrieve proposal history per token | ✅ | Token ID |
| `get_vote_history` | Get votes on a specific proposal | ✅ | Token ID, Proposal ID |
| `get_execution_history` | Global execution trail | ✅ | None |
| `get_slashing_history` | Penalty/violation records | ✅ | Target, Role |
| `record_slashing` | Record slashing event | N/A | Admin only |

---

## Quick Start

### Basic Proposal Query

```typescript
// TypeScript/JavaScript Example
const proposals = await contract.query.get_proposal_history(
  'TOKEN_ID',
  {
    offset: 0,      // Start from beginning
    limit: 20,      // Get 20 items
    sort_ascending: false  // Newest first
  }
);

console.log(`Total: ${proposals.pagination.total_count}`);
console.log(`Returned: ${proposals.pagination.returned_count}`);
console.log(`Has more: ${proposals.pagination.has_more}`);
```

### Vote History with Pagination

```rust
// Rust Example (off-chain query)
let params = PaginationParams {
    offset: 0,
    limit: 50,
    sort_ascending: true,  // Chronological order
};

let vote_history = contract.get_vote_history(
    token_id,
    proposal_id,
    params
);

for vote in &vote_history.entries {
    println!("Voter: {:?}, Support: {}, Weight: {}", 
             vote.voter, vote.support, vote.vote_weight);
}
```

### Slashing History with Filters

```typescript
// Get all oracle provider slashings
const slashings = await contract.query.get_slashing_history(
  null,  // No target filter
  'OracleProvider',  // Filter by role
  {
    offset: 0,
    limit: 100,
    sort_ascending: true
  }
);
```

---

## Common Patterns

### Pattern 1: Load All Data (with pagination)

```typescript
async function loadAllHistory(contract, tokenId) {
  let allEntries = [];
  let offset = 0;
  const limit = 100;  // Max allowed
  
  while (true) {
    const result = await contract.query.get_proposal_history(tokenId, {
      offset,
      limit,
      sort_ascending: true
    });
    
    allEntries.push(...result.entries);
    
    if (!result.pagination.has_more) {
      break;
    }
    
    offset += limit;
  }
  
  return allEntries;
}
```

### Pattern 2: Infinite Scroll

```typescript
// React Example with infinite scroll
function ProposalHistory({ tokenId }) {
  const [entries, setEntries] = useState([]);
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);

  const loadMore = async () => {
    if (loading || !hasMore) return;
    
    setLoading(true);
    const result = await contract.query.get_proposal_history(tokenId, {
      offset: page * 20,
      limit: 20,
      sort_ascending: false
    });
    
    setEntries(prev => [...prev, ...result.entries]);
    setHasMore(result.pagination.has_more);
    setPage(prev => prev + 1);
    setLoading(false);
  };

  return (
    <div>
      {entries.map(entry => (
        <ProposalCard key={entry.proposal_id} entry={entry} />
      ))}
      {hasMore && (
        <button onClick={loadMore} disabled={loading}>
          {loading ? 'Loading...' : 'Load More'}
        </button>
      )}
    </div>
  );
}
```

### Pattern 3: Real-time Dashboard

```typescript
// Poll for latest data every 30 seconds
useEffect(() => {
  async function fetchLatest() {
    const result = await contract.query.get_execution_history({
      offset: 0,
      limit: 10,
      sort_ascending: false
    });
    
    setRecentExecutions(result.entries);
  }
  
  fetchLatest();
  const interval = setInterval(fetchLatest, 30000);
  
  return () => clearInterval(interval);
}, []);
```

### Pattern 4: Audit Trail Export

```typescript
// Export complete history for audit
async function exportAuditData(tokenId) {
  const allProposals = await loadAllHistory(contract, tokenId);
  
  const auditReport = {
    exportDate: new Date().toISOString(),
    tokenId,
    totalProposals: allProposals.length,
    proposals: allProposals.map(p => ({
      id: p.proposal_id,
      status: p.status,
      created: new Date(p.created_at),
      executed: p.executed_at ? new Date(p.executed_at) : null,
      forVotes: p.for_votes,
      againstVotes: p.against_votes,
      quorum: p.quorum,
    }))
  };
  
  // Download as JSON
  const blob = new Blob([JSON.stringify(auditReport, null, 2)], 
                        { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `audit-${tokenId}-${Date.now()}.json`;
  a.click();
}
```

---

## Parameter Cheat Sheet

### PaginationParams

```typescript
{
  offset: 0,           // Start from first item (0-indexed)
  limit: 20,           // Number of items to return (max: 100)
  sort_ascending: true // true = oldest first, false = newest first
}
```

### Recommended Limits

| Use Case | Recommended Limit | Sort Direction |
|----------|------------------|----------------|
| Dashboard widget | 5-10 | Descending |
| Full table view | 20-50 | Either |
| Data export | 100 (max) | Ascending |
| Mobile app | 10-20 | Descending |
| Audit query | 100 (max) | Ascending |

---

## Response Structure

All paginated responses follow this pattern:

```typescript
{
  entries: [ /* array of history items */ ],
  pagination: {
    total_count: 150,      // Total items available
    returned_count: 20,    // Items in this response
    offset: 0,            // Current offset
    limit: 20,            // Current limit
    has_more: true        // More items available?
  }
}
```

---

## Error Handling

```typescript
try {
  const result = await contract.query.get_proposal_history(tokenId, params);
  
  if (result.pagination.total_count === 0) {
    console.log('No history available');
    return;
  }
  
  // Process entries
  console.log(result.entries);
  
} catch (error) {
  if (error.message.includes('TokenNotFound')) {
    console.error('Invalid token ID');
  } else if (error.message.includes('Unauthorized')) {
    console.error('Access denied');
  } else {
    console.error('Query failed:', error);
  }
}
```

---

## Gas Cost Estimates

| Operation | Approximate Gas Cost |
|-----------|---------------------|
| get_proposal_history (20 items) | ~45,000 |
| get_vote_history (20 items) | ~35,000 |
| get_execution_history (20 items) | ~40,000 |
| get_slashing_history (20 items) | ~55,000 |
| record_slashing | ~25,000 |

*Costs vary based on network conditions and data size*

---

## Best Practices

### DO ✅

- Use reasonable limits (20-50 for UI, 100 for exports)
- Implement caching for static historical data
- Show pagination metadata to users
- Allow users to toggle sort direction
- Handle empty states gracefully
- Implement loading states
- Check `has_more` before requesting next page

### DON'T ❌

- Request more than 100 items in a single query
- Assume ordering without explicit sort parameter
- Ignore pagination metadata
- Make unlimited sequential queries (rate limit!)
- Query on every render (use proper state management)
- Forget to handle errors
- Display stale data (implement refresh mechanism)

---

## Testing Commands

### Query Testing (using polkadot{.js})

```javascript
// Test proposal history
const proposals = await api.query.propertyToken.getProposalHistory(
  tokenId,
  { offset: 0, limit: 10, sortAscending: true }
);
console.log(proposals);

// Test vote history
const votes = await api.query.propertyToken.getVoteHistory(
  tokenId,
  proposalId,
  { offset: 0, limit: 50, sortAscending: false }
);
console.log(votes);
```

### Local Node Testing

```bash
# Deploy contract to local node
cargo contract instantiate --suri //Alice

# Call history API
cargo contract call get_proposal_history \
  --args "1" '{ "offset": 0, "limit": 20, "sort_ascending": false }'
```

---

## Integration Checklist

Before deploying to production:

- [ ] Test with empty history
- [ ] Test with single item
- [ ] Test with large datasets (1000+ items)
- [ ] Verify pagination boundaries
- [ ] Test both sort directions
- [ ] Validate has_more flag accuracy
- [ ] Check gas costs for max page size
- [ ] Implement error handling
- [ ] Add loading states
- [ ] Set up caching strategy
- [ ] Configure rate limiting
- [ ] Monitor query performance
- [ ] Document for your team

---

## Support Resources

- **Full Documentation**: [enterprise-history-apis.md](./enterprise-history-apis.md)
- **Implementation Details**: [IMPLEMENTATION_SUMMARY_HISTORY_APIS.md](./IMPLEMENTATION_SUMMARY_HISTORY_APIS.md)
- **Contract Reference**: [contracts.md](./contracts.md)
- **Architecture Overview**: [architecture.md](./architecture.md)

---

## Quick Troubleshooting

| Problem | Solution |
|---------|----------|
| Empty results | Check token_id, verify offset < total_count |
| Slow queries | Reduce limit, implement caching |
| Wrong order | Verify sort_ascending parameter |
| Missing pages | Check has_more flag, increment offset correctly |
| High gas cost | Decrease limit parameter |

---

**Last Updated**: March 26, 2026  
**Version**: 1.0.0
