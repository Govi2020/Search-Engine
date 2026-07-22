import { motion } from 'framer-motion'
import { useState } from 'react'
import './App.css'

const suggestions = ['design systems', 'semantic search', 'AI agents', 'product analytics']

function App() {
  const [query, setQuery] = useState('design systems')
  const [submittedQuery, setSubmittedQuery] = useState('')
  const [isSearching, setIsSearching] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [visibleResults, setVisibleResults] = useState([])

  const startSearch = async (nextQuery) => {
    const normalized = nextQuery.trim() || 'design systems'
    setQuery(normalized)
    setSubmittedQuery(normalized)
    setIsLoading(true)
    setIsSearching(true)

    try {
      const response = await fetch(`http://localhost:3000/?query=${encodeURIComponent(normalized)}`)
      const data = await response.json()
      setVisibleResults(Array.isArray(data) ? data : [])
    } catch (error) {
      console.error('Search request failed:', error)
      setVisibleResults([])
    } finally {
      setIsLoading(false)
    }
  }

  const handleSubmit = (event) => {
    event.preventDefault()
    startSearch(query)
  }

  const getDisplayUrl = (value) => {
    if (!value) return 'website'
    return value.replace(/^https?:\/\//i, '').replace(/\/$/, '')
  }

  const getResultHref = (value) => {
    if (!value) return '#'
    const trimmed = value.trim()
    if (/^https?:\/\//i.test(trimmed)) return trimmed
    return `https://${trimmed}`
  }

  return (
    <div className={`app-shell ${isSearching ? 'results-open' : ''}`}>
      <main className="main-stage">
        <section className={`hero-panel ${isSearching ? 'hero-panel-compact' : ''}`}>
          {!isSearching && (
            <motion.div
              className="landing-brand"
              initial={false}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.45, ease: [0.2, 0.8, 0.2, 1] }}
            >
              <div className="brand-badge">G</div>
              <span>Googol</span>
            </motion.div>
          )}

          <motion.form
            className={`search-form ${isSearching ? 'search-form-compact' : ''}`}
            onSubmit={handleSubmit}
            initial={false}
            animate={isSearching ? { y: -10, scale: 1 } : { y: 0, scale: 1 }}
            transition={{ duration: 0.65, ease: [0.2, 0.8, 0.2, 1] }}
          >
            <motion.label
              className={`search-input-wrap ${isSearching ? 'search-input-wrap-compact' : ''}`}
              htmlFor="search"
              initial={false}
              animate={isSearching ? { y: -2, scale: 1.01, boxShadow: '0 16px 44px rgba(0, 0, 0, 0.3)' } : { y: 0, scale: 1, boxShadow: '0 8px 30px rgba(0, 0, 0, 0.2)' }}
              transition={{ duration: 0.65, ease: [0.2, 0.8, 0.2, 1] }}
            >
              <motion.div
                className="brand-inline"
                initial={false}
                animate={isSearching ? { opacity: 1, width: 36, marginRight: 8, scale: 1 } : { opacity: 0, width: 0, marginRight: 0, scale: 0.9 }}
                transition={{ duration: 0.45, ease: [0.2, 0.8, 0.2, 1] }}
              >
                <div className="brand-badge">N</div>
              </motion.div>
              <span className="search-icon">⌕</span>
              <input
                id="search"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Search"
              />
            </motion.label>
            <motion.button
              type="submit"
              whileHover={{ scale: 1.02, y: -1 }}
              whileTap={{ scale: 0.98 }}
            >
              Search
            </motion.button>
          </motion.form>

          {!isSearching && (
            <div className="suggestions-row">
              {suggestions.map((suggestion) => (
                <button
                  key={suggestion}
                  className="chip"
                  type="button"
                  onClick={() => startSearch(suggestion)}
                >
                  {suggestion}
                </button>
              ))}
            </div>
          )}
        </section>

        {isSearching && (
          <section className="results-panel">
            <div className="results-header">
              <span className="results-label">Results</span>
              <span className="results-count">{isLoading ? 'Loading…' : `${visibleResults.length} results`}</span>
            </div>

            <div className="results-list">
              {isLoading ? (
                Array.from({ length: 4 }).map((_, index) => (
                  <div className="skeleton-card" key={`skeleton-${index}`}>
                    <div className="skeleton-line skeleton-line-short" />
                    <div className="skeleton-line skeleton-line-medium" />
                    <div className="skeleton-line skeleton-line-long" />
                  </div>
                ))
              ) : (
                visibleResults.map((item) => (
                  <article className="result-item" key={`${item.title}-${item.url}`}>
                    <a className="result-url" href={getResultHref(item.url)} target="_blank" rel="noreferrer">
                      {getDisplayUrl(item.url)}
                    </a>
                    <a className="result-title-link" href={getResultHref(item.url)} target="_blank" rel="noreferrer">
                      {item.title}
                    </a>
                    <p className="result-description">{item.description}</p>
                  </article>
                ))
              )}
            </div>
          </section>
        )}
      </main>
    </div>
  )
}

export default App
