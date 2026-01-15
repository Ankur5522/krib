import { TrendingUp, X, Activity } from "lucide-react";
import { useState, useEffect } from "react";
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
}: CityStatsProps) {
  const [cityViews, setCityViews] = useState<CityView[]>([]);
  const [loading, setLoading] = useState(true);
  const [lastFetchTime, setLastFetchTime] = useState<number>(0);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  useEffect(() => {
    fetchCityStats();
    // Refresh stats every 30 seconds for live updates
    const interval = setInterval(fetchCityStats, 30 * 1000);
    return () => clearInterval(interval);
  }, []);

  const fetchCityStats = async () => {
    // Check if we have cached data that's still fresh
    const now = Date.now();
    if (now - lastFetchTime < CACHE_DURATION) {
      const cached = localStorage.getItem(CACHE_KEY);
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
      const data = await apiGet<CityView[]>("/api/stats/cities");
      setCityViews(data);
      setLastFetchTime(now);
      setLastUpdated(new Date());

      // Cache the data
      try {
        localStorage.setItem(CACHE_KEY, JSON.stringify(data));
      } catch {
        // localStorage might be full, ignore
      }
    } catch (error) {
      console.error("Failed to fetch city stats:", error);

      // Try to use stale cache if fetch fails
      const cached = localStorage.getItem(CACHE_KEY);
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
  };

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
            {cityViews.map((city) => (
              <div
                key={city.city}
                className={`${theme.bgTertiary} rounded-lg p-3 border ${theme.border} hover:border-blue-500 transition-all cursor-default shadow-sm hover:shadow-md`}
              >
                <div className={`flex items-center justify-between mb-2`}>
                  <span className={`text-sm font-bold ${theme.text}`}>
                    {city.city}
                  </span>
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
                    <span className="font-semibold">{city.daily_average}</span>
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
            ))}
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
