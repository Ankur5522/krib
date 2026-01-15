import { TrendingUp, X, Activity } from "lucide-react";
import { useState, useEffect, useCallback } from "react";
import { apiGet } from "../lib/api";
import type { Theme } from "./MessageList";

interface CityView {
  city: string;
  views: number;
  daily_average: number;
}

interface CityStatsProps {
  theme: Theme;
  isOpen?: boolean;
  onClose?: () => void;
  isMobile?: boolean;
  currentCity?: string;
}

const CACHE_KEY = "krib_city_stats_cache";
const CACHE_DURATION = 5 * 60 * 1000; // 5 minutes

declare global {
  interface CSSStyleDeclaration {
    scrollbarWidth: string;
    scrollbarColor: string;
  }
}

export function CityStats({
  theme,
  isOpen = true,
  onClose,
  isMobile = false,
  currentCity,
}: CityStatsProps) {
  const [cityViews, setCityViews] = useState<CityView[]>([]);
  const [loading, setLoading] = useState(true);
  const [lastFetchTime, setLastFetchTime] = useState<number>(0);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  // Process city views to show current city at top
  const displayCityViews = () => {
    if (!currentCity || cityViews.length === 0) return cityViews;

    // Sort with current city at top, keep the rest sorted by views
    return [...cityViews].sort((a, b) => {
      const aIsCurrentCity = a.city.toLowerCase() === currentCity.toLowerCase();
      const bIsCurrentCity = b.city.toLowerCase() === currentCity.toLowerCase();

      if (aIsCurrentCity) return -1;
      if (bIsCurrentCity) return 1;
      return b.views - a.views;
    });
  };

  const fetchCityStats = useCallback(async () => {
    // Include current city in cache key so each city has its own cache
    const cacheKey = currentCity ? `${CACHE_KEY}_${currentCity}` : CACHE_KEY;

    // Check if we have cached data that's still fresh
    const now = Date.now();
    if (now - lastFetchTime < CACHE_DURATION) {
      const cached = localStorage.getItem(cacheKey);
      if (cached) {
        try {
          const data = JSON.parse(cached);
          setCityViews(data);
          setLoading(false);
          return;
        } catch {
          // Invalid cache, continue with fetch
        }
      }
    }

    try {
      // Pass current city as query parameter so backend can include it
      const url = currentCity
        ? `/api/stats/cities?current_city=${encodeURIComponent(currentCity)}`
        : "/api/stats/cities";
      const data = await apiGet<CityView[]>(url);
      setCityViews(data);
      setLastFetchTime(now);
      setLastUpdated(new Date());

      // Cache the data with city-specific key
      try {
        localStorage.setItem(cacheKey, JSON.stringify(data));
      } catch {
        // localStorage might be full, ignore
      }
    } catch (error) {
      console.error("Failed to fetch city stats:", error);

      // Try to use stale cache if fetch fails
      const cached = localStorage.getItem(cacheKey);
      if (cached) {
        try {
          const data = JSON.parse(cached);
          setCityViews(data);
        } catch {
          // Invalid cache
        }
      }
    } finally {
      setLoading(false);
    }
  }, [currentCity, lastFetchTime]);

  useEffect(() => {
    // Reset cache time when city changes to force fresh fetch
    setLastFetchTime(0);
    fetchCityStats();
    // Refresh stats every 30 seconds for live updates
    const interval = setInterval(fetchCityStats, 30 * 1000);
    return () => clearInterval(interval);
  }, [currentCity, fetchCityStats]);

  if (isMobile && !isOpen) {
    return null;
  }

  const content = (
    <div
      className={`space-y-3 overflow-y-auto ${theme.bgSecondary} ${
        isMobile ? `fixed inset-0 z-50 p-4` : `h-full p-4`
      }`}
    >
      {isMobile && onClose && (
        <div className="flex items-center justify-between mb-4 sticky top-0">
          <h2 className={`text-lg font-bold ${theme.text}`}>City Views</h2>
          <button
            onClick={onClose}
            className={`p-2 rounded-lg ${theme.accentSoft} hover:opacity-80 transition-all`}
          >
            <X className="w-5 h-5" />
          </button>
        </div>
      )}

      {!isMobile && (
        <div
          className={`flex items-center gap-2 px-3 mb-3 sticky top-0 pt-2 pb-2 border-b ${theme.border} ${theme.bgTertiary}`}
        >
          <Activity className="w-4 h-4 text-blue-500" />
          <h2
            className={`text-xs font-bold uppercase tracking-wider ${theme.text}`}
          >
            Live City Stats
          </h2>
        </div>
      )}

      {loading && cityViews.length === 0 ? (
        <div className={`text-center py-8 ${theme.textMuted} text-sm`}>
          Loading...
        </div>
      ) : cityViews.length === 0 ? (
        <div className={`text-center py-8 ${theme.textMuted} text-sm`}>
          No data yet
        </div>
      ) : (
        <>
          <div className="space-y-2 flex-1">
            {displayCityViews().map((city) => {
              const isCurrentCity =
                currentCity &&
                city.city.toLowerCase() === currentCity.toLowerCase();
              return (
                <div
                  key={city.city}
                  className={`${theme.bgTertiary} rounded-lg p-3 border ${
                    isCurrentCity
                      ? "border-blue-500 ring-1 ring-blue-500/50"
                      : theme.border
                  } hover:border-blue-500 transition-all cursor-default shadow-sm hover:shadow-md`}
                >
                  <div className={`flex items-center justify-between mb-2`}>
                    <div className="flex items-center gap-2">
                      <span className={`text-sm font-bold ${theme.text}`}>
                        {city.city}
                      </span>
                      {isCurrentCity && (
                        <span className="text-xs px-2 py-0.5 rounded-full bg-blue-500 text-white">
                          You
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-1.5">
                      <TrendingUp className="w-3.5 h-3.5 text-emerald-500" />
                      <span className={`text-xs font-bold text-emerald-500`}>
                        {city.views}
                      </span>
                    </div>
                  </div>
                  <div className="flex items-center justify-between">
                    <p className={`text-xs ${theme.textMuted}`}>
                      <span className="opacity-70 text-xs">Daily: </span>
                      <span className="font-semibold">
                        {city.daily_average}
                      </span>
                    </p>
                    <div className="w-16 h-1.5 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-linear-to-r from-blue-500 to-cyan-500 rounded-full"
                        style={{
                          width: `${Math.min(100, (city.views / 50) * 100)}%`,
                        }}
                      />
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
          {lastUpdated && !isMobile && (
            <p
              className={`text-xs ${theme.textMuted} text-center pt-2 border-t ${theme.border}`}
            >
              Updated {Math.round((Date.now() - lastUpdated.getTime()) / 1000)}s
              ago
            </p>
          )}
        </>
      )}
    </div>
  );

  if (isMobile) {
    return content;
  }

  return (
    <div
      className={`city-stats-scroll w-72 max-h-96 ${theme.bgSecondary} border ${theme.border} rounded-xl flex flex-col overflow-hidden shadow-lg`}
      style={{
        backdropFilter: "blur(8px)",
      }}
    >
      {content}
    </div>
  );
}
