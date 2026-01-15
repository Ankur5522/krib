import { MapPin, TrendingUp, X } from "lucide-react";
import { useState, useEffect } from "react";
import { apiGet } from "../lib/api";

interface CityView {
  city: string;
  views: number;
  daily_average: number;
}

interface CityStatsProps {
  theme: any;
  isOpen?: boolean;
  onClose?: () => void;
  isMobile?: boolean;
}

const CACHE_KEY = "krib_city_stats_cache";
const CACHE_DURATION = 5 * 60 * 1000; // 5 minutes

export function CityStats({
  theme,
  isOpen = true,
  onClose,
  isMobile = false,
}: CityStatsProps) {
  const [cityViews, setCityViews] = useState<CityView[]>([]);
  const [loading, setLoading] = useState(true);
  const [lastFetchTime, setLastFetchTime] = useState<number>(0);

  useEffect(() => {
    fetchCityStats();
    // Refresh stats every 2 minutes instead of 1 to avoid rate limiting
    const interval = setInterval(fetchCityStats, 2 * 60 * 1000);
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
      className={`space-y-4 ${
        isMobile
          ? `fixed inset-0 z-50 ${theme.bg} overflow-auto p-4`
          : `h-full overflow-y-auto p-4 ${theme.bgSecondary}`
      }`}
    >
      {isMobile && onClose && (
        <div className="flex items-center justify-between mb-4">
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
        <div className={`flex items-center gap-2 px-2 mb-4`}>
          <MapPin className="w-5 h-5" />
          <h2 className={`text-sm font-bold ${theme.text}`}>City Views</h2>
        </div>
      )}

      {loading && cityViews.length === 0 ? (
        <div
          className={`text-center py-8 ${theme.textMuted} text-sm`}
        >
          Loading...
        </div>
      ) : cityViews.length === 0 ? (
        <div
          className={`text-center py-8 ${theme.textMuted} text-sm`}
        >
          No data yet
        </div>
      ) : (
        <div className="space-y-2">
          {cityViews.map((city) => (
            <div
              key={city.city}
              className={`${theme.bgTertiary} rounded-lg p-3 border ${theme.border} hover:opacity-80 transition-all cursor-default`}
            >
              <div
                className={`flex items-center justify-between mb-1`}
              >
                <span className={`text-sm font-medium ${theme.text}`}>
                  {city.city}
                </span>
                <div className="flex items-center gap-1">
                  <TrendingUp className="w-3 h-3 text-green-500" />
                  <span className={`text-xs font-semibold text-green-500`}>
                    {city.views}
                  </span>
                </div>
              </div>
              <p className={`text-xs ${theme.textMuted}`}>
                Avg: {city.daily_average}/day
              </p>
            </div>
          ))}
        </div>
      )}
    </div>
  );

  if (isMobile) {
    return content;
  }

  return (
    <div
      className={`w-60 ${theme.bgSecondary} border-r ${theme.border} flex flex-col hidden lg:flex`}
    >
      {content}
    </div>
  );
}

