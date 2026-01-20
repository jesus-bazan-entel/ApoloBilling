import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import Layout from './components/Layout'
import Dashboard from './pages/Dashboard'
import ActiveCalls from './pages/ActiveCalls'
import CDR from './pages/CDR'
import Accounts from './pages/Accounts'
import Balance from './pages/Balance'
import Zones from './pages/Zones'
import Rates from './pages/Rates'

// Create a client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
      staleTime: 5000,
    },
  },
})

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Layout>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/calls" element={<ActiveCalls />} />
            <Route path="/cdr" element={<CDR />} />
            <Route path="/accounts" element={<Accounts />} />
            <Route path="/balance" element={<Balance />} />
            <Route path="/zones" element={<Zones />} />
            <Route path="/rates" element={<Rates />} />
          </Routes>
        </Layout>
      </BrowserRouter>
    </QueryClientProvider>
  )
}

export default App
