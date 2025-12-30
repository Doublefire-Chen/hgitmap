import { useState, useEffect, useCallback } from 'react';
import apiClient from '../api/client';
import './Heatmap.css';

function Heatmap({ platformFilter = 'all', setPlatformFilter, username = null, isPublic = false }) {
  const [contributions, setContributions] = useState([]);
  const [_stats, setStats] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [viewMode, setViewMode] = useState('rolling'); // 'rolling' or 'year'
  const [year, setYear] = useState(new Date().getFullYear());
  const [platforms, setPlatforms] = useState([]);

  // Load available platforms on mount (only for authenticated users viewing their own profile)
  useEffect(() => {
    if (!isPublic) {
      const loadPlatforms = async () => {
        try {
          const platformList = await apiClient.listPlatforms();
          setPlatforms(platformList);
        } catch (err) {
          console.error('Failed to load platforms:', err);
        }
      };
      loadPlatforms();
    }
  }, [isPublic]);

  const loadData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      let fromDate, toDate;

      if (viewMode === 'rolling') {
        // Rolling 365 days: match GitHub's behavior
        // GitHub completes the first week by starting from the Sunday before
        const today = new Date();

        // Calculate 365-day window start (364 days back + today = 365 days)
        const windowStart = new Date(today);
        windowStart.setDate(windowStart.getDate() - 364);

        // Find the Sunday before (or on) the window start
        const dayOfWeek = windowStart.getDay(); // 0 = Sunday, 6 = Saturday
        const from = new Date(windowStart);
        if (dayOfWeek !== 0) {
          from.setDate(from.getDate() - dayOfWeek); // Go back to Sunday
        }

        // Format in local timezone to avoid UTC conversion issues
        const formatLocalDate = (date) => {
          const year = date.getFullYear();
          const month = String(date.getMonth() + 1).padStart(2, '0');
          const day = String(date.getDate()).padStart(2, '0');
          return `${year}-${month}-${day}`;
        };

        fromDate = formatLocalDate(from);
        toDate = formatLocalDate(today);

        console.log(`📅 [loadData] Rolling mode: 365-day window starts ${formatLocalDate(windowStart)}`);
        console.log(`📅 [loadData] Fetching from Sunday ${fromDate} to ${toDate}`);
      } else {
        // Calendar year mode: Jan 1 to Dec 31 of selected year
        fromDate = `${year}-01-01`;
        toDate = `${year}-12-31`;

        console.log(`📅 [loadData] Year mode: ${fromDate} to ${toDate} (year ${year})`);
      }

      // Use public or authenticated API based on isPublic flag
      const [contributionsData, statsData] = await Promise.all([
        isPublic
          ? apiClient.getUserContributions(username, fromDate, toDate, platformFilter !== 'all' ? platformFilter : null)
          : apiClient.getContributions(fromDate, toDate, platformFilter !== 'all' ? platformFilter : null),
        isPublic
          ? apiClient.getUserStats(username)
          : apiClient.getContributionStats(),
      ]);

      console.log(`📊 [loadData] Received ${contributionsData.contributions?.length || 0} contribution days`);
      console.log(`📊 [loadData] First 5 dates:`, contributionsData.contributions?.slice(0, 5).map(c => c.date));
      console.log(`📊 [loadData] Last 5 dates:`, contributionsData.contributions?.slice(-5).map(c => c.date));

      setContributions(contributionsData.contributions || []);
      setStats(statsData);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }, [viewMode, year, platformFilter, username, isPublic]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const getWeeksInYear = () => {
    const weeks = [];
    let startDate, endDate, windowStart;

    if (viewMode === 'rolling') {
      // Rolling 365 days mode (364 days back + today = 365 days total)
      const today = new Date();
      today.setHours(23, 59, 59, 999);

      // The actual 365-day window starts here
      const actualWindowStart = new Date(today);
      actualWindowStart.setDate(actualWindowStart.getDate() - 364);
      actualWindowStart.setHours(0, 0, 0, 0);
      windowStart = actualWindowStart;

      // But we fetch from the Sunday before to complete the first week
      const dayOfWeek = actualWindowStart.getDay();
      const from = new Date(actualWindowStart);
      if (dayOfWeek !== 0) {
        from.setDate(from.getDate() - dayOfWeek);
      }
      from.setHours(0, 0, 0, 0);

      startDate = from;
      endDate = today;

      console.log('=== Heatmap Date Range Debug (Rolling Mode) ===');
      console.log('365-day window starts:', windowStart.toDateString());
      console.log('Fetch from Sunday:', startDate.toDateString());
      console.log('End Date (today):', endDate.toDateString());
    } else {
      // Calendar year mode
      startDate = new Date(year, 0, 1);
      startDate.setHours(0, 0, 0, 0);
      windowStart = startDate; // In year mode, window start = fetch start

      const today = new Date();
      today.setHours(23, 59, 59, 999);

      endDate = new Date(year, 11, 31);
      endDate.setHours(23, 59, 59, 999);
      if (year === today.getFullYear()) {
        endDate = today;
      }

      console.log('=== Heatmap Date Range Debug (Year Mode) ===');
      console.log('Year:', year);
      console.log('Start Date (Jan 1):', startDate.toDateString());
      console.log('End Date:', endDate.toDateString());
      console.log('Jan 1 is a:', ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'][startDate.getDay()]);
    }

    // Start from the Sunday of the week containing January 1st
    let currentDate = new Date(startDate);
    const startDay = startDate.getDay(); // 0 = Sunday, 6 = Saturday

    // Go back to the Sunday of the first week
    if (startDay !== 0) {
      currentDate.setDate(currentDate.getDate() - startDay);
    }

    console.log('First Sunday:', currentDate.toDateString());

    let weekIndex = 0;
    while (currentDate <= endDate) {
      const week = [];
      for (let i = 0; i < 7; i++) {
        const date = new Date(currentDate);
        date.setDate(date.getDate() + i);
        date.setHours(0, 0, 0, 0); // Normalize to midnight for comparison

        // Format date in local timezone (not UTC) to avoid timezone offset issues
        const dateYear = date.getFullYear();
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const day = String(date.getDate()).padStart(2, '0');
        const dateStr = `${dateYear}-${month}-${day}`;
        const contribution = contributions.find((c) => c.date === dateStr);
        const count = contribution ? contribution.count : 0;

        // Determine if we should show this day based on view mode
        let shouldInclude;
        if (viewMode === 'rolling') {
          // Rolling mode: show all dates from startDate (Sunday) to endDate (today)
          // This includes dates before the 365-day window (they'll show as empty)
          shouldInclude = date >= startDate && date <= endDate;
        } else {
          // Year mode: show only dates in the selected year AND between startDate and endDate
          shouldInclude = dateYear === year && date >= startDate && date <= endDate;
        }

        // Debug first week
        if (weekIndex === 0) {
          const inWindow = viewMode === 'rolling' ? date >= windowStart : true;
          console.log(`Day ${i}:`, date.toDateString(),
            '-> dateStr:', dateStr,
            '| Mode:', viewMode,
            '| >= Start?', date >= startDate,
            '| <= End?', date <= endDate,
            '| In window?', inWindow,
            '| Should include?', shouldInclude);
        }

        if (shouldInclude) {
          week.push({
            date: dateStr,
            count,
            month: date.getMonth(),
            day: date.getDay(),
          });
        } else {
          week.push(null); // Empty cell for days outside the date range
        }
      }
      weeks.push(week);
      currentDate.setDate(currentDate.getDate() + 7);
      weekIndex++;
    }

    console.log('Total weeks:', weeks.length);
    console.log('First week contents:', weeks[0]);
    console.log('First week days:');
    weeks[0].forEach((day, i) => {
      if (day) {
        console.log(`  [${i}] ${day.date} - count: ${day.count}`);
      } else {
        console.log(`  [${i}] NULL (empty cell)`);
      }
    });
    console.log('================================');

    return weeks;
  };

  const calculateQuartiles = (contributions) => {
    // GitHub: Calculate quartiles from the range [0, max]
    // "quartiles of the normal distribution over the range [0, max(v)]"
    const nonZeroCounts = contributions
      .map(c => c.count)
      .filter(count => count > 0);

    if (nonZeroCounts.length === 0) {
      return { q1: 0, q2: 0, q3: 0, max: 0 };
    }

    const max = Math.max(...nonZeroCounts);

    // Divide the max into quartiles (not percentiles of the data)
    const q1 = max * 0.25;
    const q2 = max * 0.50;
    const q3 = max * 0.75;

    console.log('📊 [Quartiles] Non-zero days:', nonZeroCounts.length, '| Max:', max);
    console.log('📊 [Quartiles] Q1:', q1, '| Q2:', q2, '| Q3:', q3);
    console.log('📊 [Quartiles] Using range-based quartiles: [0, max] divided into 4 parts');

    return { q1, q2, q3, max };
  };

  const getIntensityLevel = (count, quartiles) => {
    if (count === 0) return 0;

    // GitHub's quartile-based algorithm:
    // Level 1: up to Q1 (inclusive)
    // Level 2: above Q1, up to Q2 (inclusive)
    // Level 3: above Q2, up to Q3 (inclusive)
    // Level 4: above Q3
    if (count <= quartiles.q1) return 1;
    if (count <= quartiles.q2) return 2;
    if (count <= quartiles.q3) return 3;
    return 4;
  };

  const getOrdinalSuffix = (day) => {
    if (day > 3 && day < 21) return 'th';
    switch (day % 10) {
      case 1: return 'st';
      case 2: return 'nd';
      case 3: return 'rd';
      default: return 'th';
    }
  };

  const formatTooltipDate = (date) => {
    const monthNames = ['January', 'February', 'March', 'April', 'May', 'June',
      'July', 'August', 'September', 'October', 'November', 'December'];
    const month = monthNames[date.getMonth()];
    const day = date.getDate();
    const suffix = getOrdinalSuffix(day);
    return `${month} ${day}${suffix}`;
  };

  const getMonthLabels = (weeks) => {
    const monthLabels = [];
    let lastMonth = -1;

    weeks.forEach((week, weekIndex) => {
      // Find the first valid day in the week to determine the month
      const firstValidDay = week.find(day => day !== null);

      if (firstValidDay && firstValidDay.month !== lastMonth) {
        monthLabels.push({
          month: firstValidDay.month,
          weekIndex: weekIndex,
        });
        lastMonth = firstValidDay.month;
      }
    });

    return monthLabels;
  };

  if (loading) {
    return (
      <div className="heatmap-container">
        <div className="heatmap-loading">Loading contributions...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="heatmap-container">
        <div className="heatmap-error">Error loading contributions: {error}</div>
      </div>
    );
  }

  const weeks = getWeeksInYear();
  const monthLabels = getMonthLabels(weeks);
  const totalContributions = contributions.reduce((sum, c) => sum + c.count, 0);

  // Calculate quartiles for dynamic color intensity (GitHub-style)
  const quartiles = calculateQuartiles(contributions);

  console.log('📊 Total contributions calculation:');
  console.log('  - Contribution days:', contributions.length);
  console.log('  - Total count:', totalContributions);
  console.log('  - View mode:', viewMode);
  console.log('📊 Quartiles:', quartiles);

  const monthNames = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

  return (
    <div className="heatmap-container">
      <div className="heatmap-layout">
        <div className="heatmap-main">
          {/* Header */}
          <div className="heatmap-header">
            <h2>
              {totalContributions.toLocaleString()} contributions{' '}
              {viewMode === 'rolling' ? 'in the last year' : `in ${year}`}
            </h2>
          </div>

          {/* Heatmap Grid */}
          <div className="heatmap-wrapper">
            <div className="heatmap-months">
              {monthLabels.map((label, index) => (
                <div
                  key={index}
                  className="month-label"
                  style={{ gridColumn: label.weekIndex + 1 }}
                >
                  {monthNames[label.month]}
                </div>
              ))}
            </div>

            <div className="heatmap-grid-wrapper">
              <div className="heatmap-days">
                <div className="day-label"></div>
                <div className="day-label">Mon</div>
                <div className="day-label"></div>
                <div className="day-label">Wed</div>
                <div className="day-label"></div>
                <div className="day-label">Fri</div>
                <div className="day-label"></div>
              </div>

              <div className="heatmap-grid">
                {weeks.map((week, weekIndex) => (
                  <div key={weekIndex} className="heatmap-week">
                    {week.map((day, dayIndex) => {
                      // Debug rendering for first week
                      if (weekIndex === 0) {
                        console.log(`[RENDER] Week 0, Day ${dayIndex}:`, day === null ? 'NULL' : `date=${day.date}, count=${day.count}`);
                      }

                      if (!day) {
                        return <div key={dayIndex} className="heatmap-day empty"></div>;
                      }

                      const level = getIntensityLevel(day.count, quartiles);
                      const date = new Date(day.date);
                      const tooltipDate = formatTooltipDate(date);
                      const tooltipText = day.count === 0
                        ? `No contributions on ${tooltipDate}.`
                        : `${day.count} contribution${day.count === 1 ? '' : 's'} on ${tooltipDate}.`;

                      return (
                        <div
                          key={dayIndex}
                          className={`heatmap-day level-${level}`}
                          data-count={day.count}
                          data-date={tooltipDate}
                          title={tooltipText}
                        ></div>
                      );
                    })}
                  </div>
                ))}
              </div>
            </div>

            {/* Legend */}
            <div className="heatmap-legend">
              <span className="legend-label">Less</span>
              <div className="legend-square level-0"></div>
              <div className="legend-square level-1"></div>
              <div className="legend-square level-2"></div>
              <div className="legend-square level-3"></div>
              <div className="legend-square level-4"></div>
              <span className="legend-label">More</span>
            </div>
          </div>

          {contributions.length === 0 && (
            <div className="no-contributions-message">
              <p>No contributions yet {viewMode === 'rolling' ? 'in the last year' : `for ${year}`}.</p>
              <p>Connect a platform to start tracking your contributions!</p>
            </div>
          )}
        </div>

        {/* Year Selector Sidebar */}
        <div className="year-selector-sidebar">
          {/* Platform Filter */}
          <div className="platform-filter-section">
            <label htmlFor="platform-filter" className="platform-filter-label">
              Filter by Platform:
            </label>
            <select
              id="platform-filter"
              value={platformFilter}
              onChange={(e) => setPlatformFilter(e.target.value)}
              className="platform-filter-select"
            >
              <option value="all">All Platforms</option>
              {platforms.map((platform) => (
                <option key={platform.id} value={platform.platform}>
                  {platform.platform.charAt(0).toUpperCase() + platform.platform.slice(1)}
                  {platform.platform_url ? ` (${new URL(platform.platform_url).hostname})` : ''}
                </option>
              ))}
            </select>
          </div>

          <div className="year-filter-divider"></div>

          <button
            className={`year-item ${viewMode === 'rolling' ? 'active' : ''}`}
            onClick={() => {
              setViewMode('rolling');
              setYear(new Date().getFullYear());
            }}
          >
            Last year
          </button>
          {Array.from({ length: new Date().getFullYear() - 2019 }, (_, i) => {
            const y = new Date().getFullYear() - i;
            return (
              <button
                key={y}
                className={`year-item ${viewMode === 'year' && year === y ? 'active' : ''}`}
                onClick={() => {
                  setViewMode('year');
                  setYear(y);
                }}
              >
                {y}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}

export default Heatmap;
