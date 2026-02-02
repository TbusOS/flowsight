/**
 * Performance monitoring and optimization hooks
 */
import { useEffect, useRef, useCallback, useMemo, useState } from 'react'

/**
 * Debounce a value - useful for search inputs
 */
export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value)

  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedValue(value)
    }, delay)

    return () => {
      clearTimeout(timer)
    }
  }, [value, delay])

  return debouncedValue
}

/**
 * Throttle a callback - useful for scroll/resize handlers
 */
export function useThrottle<T extends (...args: unknown[]) => unknown>(
  callback: T,
  delay: number
): T {
  const lastRun = useRef(Date.now())
  const timeoutRef = useRef<ReturnType<typeof setTimeout>>()

  return useCallback(
    ((...args: Parameters<T>) => {
      const now = Date.now()
      const elapsed = now - lastRun.current

      if (elapsed >= delay) {
        lastRun.current = now
        callback(...args)
      } else {
        // Schedule for remaining time
        if (timeoutRef.current) {
          clearTimeout(timeoutRef.current)
        }
        timeoutRef.current = setTimeout(() => {
          lastRun.current = Date.now()
          callback(...args)
        }, delay - elapsed)
      }
    }) as T,
    [callback, delay]
  )
}

// Check if we're in development mode
const isDev = typeof window !== 'undefined' && 
  (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1')

/**
 * Track component render count (dev only)
 */
export function useRenderCount(componentName: string): number {
  const renderCount = useRef(0)
  renderCount.current++

  useEffect(() => {
    if (isDev) {
      console.debug(`[Render] ${componentName}: ${renderCount.current}`)
    }
  })

  return renderCount.current
}

/**
 * Measure component mount/update time
 */
export function usePerformanceMeasure(
  componentName: string,
  enabled = isDev
) {
  const startTime = useRef<number>(0)
  const measureCount = useRef(0)

  useEffect(() => {
    if (!enabled) return

    startTime.current = performance.now()

    return () => {
      const duration = performance.now() - startTime.current
      measureCount.current++
      
      // Only log slow renders (> 16ms = dropped frame)
      if (duration > 16) {
        console.warn(
          `[Perf] ${componentName} slow render #${measureCount.current}: ${duration.toFixed(2)}ms`
        )
      }
    }
  })
}

/**
 * Lazy initialization for expensive computations
 */
export function useLazyInit<T>(factory: () => T): T {
  const ref = useRef<{ value: T; initialized: boolean }>({
    value: undefined as T,
    initialized: false,
  })

  if (!ref.current.initialized) {
    ref.current.value = factory()
    ref.current.initialized = true
  }

  return ref.current.value
}

/**
 * Previous value hook - useful for comparing changes
 */
export function usePrevious<T>(value: T): T | undefined {
  const ref = useRef<T>()

  useEffect(() => {
    ref.current = value
  }, [value])

  return ref.current
}

/**
 * Stable callback that doesn't change reference
 */
export function useStableCallback<T extends (...args: unknown[]) => unknown>(
  callback: T
): T {
  const callbackRef = useRef(callback)
  
  useEffect(() => {
    callbackRef.current = callback
  }, [callback])

  return useCallback(
    ((...args: Parameters<T>) => callbackRef.current(...args)) as T,
    []
  )
}

/**
 * Intersection observer hook for lazy loading
 */
export function useInView(
  options?: IntersectionObserverInit
): [React.RefObject<HTMLDivElement>, boolean] {
  const ref = useRef<HTMLDivElement>(null)
  const [inView, setInView] = useState(false)

  useEffect(() => {
    const element = ref.current
    if (!element) return

    const observer = new IntersectionObserver(([entry]) => {
      setInView(entry.isIntersecting)
    }, options)

    observer.observe(element)

    return () => {
      observer.disconnect()
    }
  }, [options])

  return [ref, inView]
}

/**
 * Memoize expensive computations with deep comparison
 */
export function useDeepMemo<T>(factory: () => T, deps: unknown[]): T {
  const ref = useRef<{ deps: unknown[]; value: T }>()

  if (!ref.current || !shallowEqual(ref.current.deps, deps)) {
    ref.current = {
      deps,
      value: factory(),
    }
  }

  return ref.current.value
}

function shallowEqual(a: unknown[], b: unknown[]): boolean {
  if (a.length !== b.length) return false
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false
  }
  return true
}

/**
 * Request idle callback hook for non-urgent updates
 */
export function useIdleCallback(
  callback: () => void,
  deps: unknown[]
): void {
  useEffect(() => {
    if ('requestIdleCallback' in window) {
      const id = requestIdleCallback(callback)
      return () => cancelIdleCallback(id)
    } else {
      // Fallback for Safari
      const id = setTimeout(callback, 1)
      return () => clearTimeout(id)
    }
  }, deps)
}

/**
 * Batch multiple state updates
 */
export function useBatchedUpdates() {
  const pendingUpdates = useRef<Array<() => void>>([])
  const frameId = useRef<number>()

  const scheduleUpdate = useCallback((update: () => void) => {
    pendingUpdates.current.push(update)

    if (!frameId.current) {
      frameId.current = requestAnimationFrame(() => {
        const updates = pendingUpdates.current
        pendingUpdates.current = []
        frameId.current = undefined

        // Batch all updates
        updates.forEach((fn) => fn())
      })
    }
  }, [])

  useEffect(() => {
    return () => {
      if (frameId.current) {
        cancelAnimationFrame(frameId.current)
      }
    }
  }, [])

  return scheduleUpdate
}
