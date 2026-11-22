import http from 'k6/http';
import { check } from 'k6';

const BASE = __ENV.BASE || 'http://localhost:8000';
const QUERIES = [
  'how does hybrid retrieval work',
  'what database does cairn use',
  'how are notes chunked',
  'which embedding model is used',
  'how is tenant isolation tested',
  'what does the eval harness measure',
];

export const options = {
  vus: Number(__ENV.VUS || 1),
  duration: __ENV.DURATION || '20s',
  summaryTrendStats: ['min', 'med', 'p(95)', 'p(99)', 'max'],
};

export function setup() {
  const res = http.post(
    `${BASE}/auth/login`,
    JSON.stringify({ email: 'loadtest@test.dev', password: 'a-real-password' }),
    { headers: { 'Content-Type': 'application/json' } },
  );
  return { token: res.json('token') };
}

export default function (data) {
  const q = QUERIES[Math.floor(Math.random() * QUERIES.length)];
  const res = http.get(`${BASE}/search?q=${encodeURIComponent(q)}`, {
    headers: { Authorization: `Bearer ${data.token}` },
  });
  check(res, { 'status is 200': (r) => r.status === 200 });
}
