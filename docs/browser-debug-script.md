# Browser Console Debug Script

Copy and paste this into your browser console (F12 → Console tab) while on the analyzer page:

```javascript
// Debug script to check API client configuration
console.log('=== API Client Debug ===');
console.log('1. LocalStorage check:');
console.log('   - access_token:', localStorage.getItem('access_token') ? 'EXISTS (length: ' + localStorage.getItem('access_token').length + ')' : 'MISSING');
console.log('   - refresh_token:', localStorage.getItem('refresh_token') ? 'EXISTS' : 'MISSING');
console.log('   - user:', localStorage.getItem('user'));

console.log('\n2. Testing API directly:');

// Test company endpoint
const companyId = '10000000-0000-0000-0000-000000000001';
const token = localStorage.getItem('access_token');

fetch(`/api/v1/companies/${companyId}`, {
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  }
})
.then(r => {
  console.log('  Company API Response Status:', r.status, r.statusText);
  console.log('  Response URL:', r.url);
  return r.json();
})
.then(data => {
  console.log('  Company Data:', data);
})
.catch(err => {
  console.error('  Company API Error:', err);
});

// Test metrics endpoint
fetch(`/api/v1/companies/${companyId}/metrics?period_type=quarterly&period_count=8`, {
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  }
})
.then(r => {
  console.log('  Metrics API Response Status:', r.status, r.statusText);
  console.log('  Response URL:', r.url);
  return r.json();
})
.then(data => {
  console.log('  Metrics Data (periods):', data.periods?.length, 'periods');
  console.log('  Metrics Data (sections):', Object.keys(data.sections || {}));
})
.catch(err => {
  console.error('  Metrics API Error:', err);
});

console.log('\n3. Check Network tab for actual requests');
console.log('======================');
```

**After running this, report back:**
1. What status codes do you see? (200, 404, 401, etc.)
2. What does the "Response URL" show?
3. Do the requests succeed or fail?
