const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

class ApiClient {
  constructor() {
    this.isHandlingExpiration = false;
    this.onSessionExpired = null; // Callback for session expiration
  }

  setSessionExpiredCallback(callback) {
    this.onSessionExpired = callback;
  }
  async register(username, password) {
    const response = await fetch(`${API_BASE_URL}/auth/register`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ username, password }),
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error || 'Registration failed');
    }

    return data;
  }

  async login(username, password) {
    const response = await fetch(`${API_BASE_URL}/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ username, password }),
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error || 'Login failed');
    }

    return data;
  }

  async fetchWithAuth(url, options = {}) {
    const token = localStorage.getItem('token');

    console.log(`🌐 [API] ${options.method || 'GET'} ${url}`);
    if (options.body) {
      console.log(`📤 [API] Request body:`, JSON.parse(options.body));
    }

    const headers = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
      console.log(`🔑 [API] Using auth token: ${token.substring(0, 20)}...`);
    }

    const fullUrl = `${API_BASE_URL}${url}`;
    console.log(`📍 [API] Full URL: ${fullUrl}`);

    const response = await fetch(fullUrl, {
      ...options,
      headers,
    });

    console.log(`📥 [API] Response status: ${response.status} ${response.statusText}`);

    if (response.status === 401) {
      console.error('❌ [API] Unauthorized - token expired');

      // Only handle expiration once, even if multiple API calls fail simultaneously
      if (!this.isHandlingExpiration) {
        this.isHandlingExpiration = true;
        localStorage.removeItem('token');
        localStorage.removeItem('user');

        // Show toast notification instead of alert
        if (this.onSessionExpired) {
          this.onSessionExpired();
        }

        // Wait 3 seconds to let user see the toast message before redirect
        setTimeout(() => {
          window.location.href = '/login';
        }, 3000);
      }

      // Return a promise that never resolves to prevent further execution
      return new Promise(() => {});
    }

    // Handle 204 No Content responses (like DELETE)
    if (response.status === 204) {
      console.log(`✅ [API] Request successful (No Content)`);
      return null;
    }

    const data = await response.json();
    console.log(`📦 [API] Response data:`, data);

    if (!response.ok) {
      console.error(`❌ [API] Request failed:`, data);
      throw new Error(data.error || 'Request failed');
    }

    console.log(`✅ [API] Request successful`);
    return data;
  }

  // Platform Management
  async connectPlatform(platform, accessToken, instanceUrl = null) {
    return this.fetchWithAuth('/platforms/connect', {
      method: 'POST',
      body: JSON.stringify({ platform, access_token: accessToken, instance_url: instanceUrl }),
    });
  }

  async listPlatforms() {
    return this.fetchWithAuth('/platforms');
  }

  async getPlatforms() {
    return this.listPlatforms();
  }

  async disconnectPlatform(platformId) {
    return this.fetchWithAuth(`/platforms/${platformId}`, {
      method: 'DELETE',
    });
  }

  async syncPlatform(platformId, allYears = false, year = null) {
    let queryParam = '';
    if (allYears) {
      queryParam = '?all_years=true';
    } else if (year) {
      queryParam = `?year=${year}`;
    }
    return this.fetchWithAuth(`/platforms/${platformId}/sync${queryParam}`, {
      method: 'POST',
    });
  }

  async updatePlatformSyncPreferences(platformId, preferences) {
    return this.fetchWithAuth(`/platforms/${platformId}/sync-preferences`, {
      method: 'PUT',
      body: JSON.stringify(preferences),
    });
  }

  // Contributions
  async getContributions(from = null, to = null, platform = null) {
    const params = new URLSearchParams();
    if (from) params.append('from', from);
    if (to) params.append('to', to);
    if (platform) params.append('platform', platform);

    const queryString = params.toString();
    const url = queryString ? `/contributions?${queryString}` : '/contributions';

    return this.fetchWithAuth(url);
  }

  async getContributionStats() {
    return this.fetchWithAuth('/contributions/stats');
  }

  // Activities
  async getActivities(from = null, to = null, limit = 50, offset = 0, platform = null) {
    const params = new URLSearchParams();
    if (from) params.append('from', from);
    if (to) params.append('to', to);
    params.append('limit', limit.toString());
    params.append('offset', offset.toString());
    if (platform) params.append('platform', platform);

    const queryString = params.toString();
    const url = `/activities?${queryString}`;

    return this.fetchWithAuth(url);
  }

  // Settings
  async getSettings() {
    return this.fetchWithAuth('/settings');
  }

  async updateSettings(settings) {
    return this.fetchWithAuth('/settings', {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
  }

  // OAuth
  async startGithubOAuth() {
    const response = await this.fetchWithAuth('/oauth/github/authorize');
    return response.authorization_url;
  }

  async startGiteaOAuth(instanceUrl) {
    const response = await this.fetchWithAuth(`/oauth/gitea/authorize?instance_url=${encodeURIComponent(instanceUrl)}`);
    return response.authorization_url;
  }

  async startGitlabOAuth(instanceUrl = null) {
    const url = instanceUrl
      ? `/oauth/gitlab/authorize?instance_url=${encodeURIComponent(instanceUrl)}`
      : '/oauth/gitlab/authorize';
    const response = await this.fetchWithAuth(url);
    return response.authorization_url;
  }

  async listOAuthInstances(platform) {
    // This is a public endpoint, but we still use fetchWithAuth for consistency
    const response = await this.fetchWithAuth(`/oauth/instances/${platform}`);
    return response;
  }

  // Admin - OAuth Apps Management
  async listOAuthApps() {
    return this.fetchWithAuth('/admin/oauth-apps');
  }

  async createOAuthApp(platform, instanceName, instanceUrl, clientId, clientSecret, isDefault = true) {
    return this.fetchWithAuth('/admin/oauth-apps', {
      method: 'POST',
      body: JSON.stringify({
        platform,
        instance_name: instanceName,
        instance_url: instanceUrl || '',
        client_id: clientId,
        client_secret: clientSecret,
        is_default: isDefault,
        is_enabled: true,
      }),
    });
  }

  async updateOAuthApp(appId, updates) {
    return this.fetchWithAuth(`/admin/oauth-apps/${appId}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    });
  }

  async deleteOAuthApp(appId) {
    return this.fetchWithAuth(`/admin/oauth-apps/${appId}`, {
      method: 'DELETE',
    });
  }

  // Heatmap Themes
  async listThemes() {
    return this.fetchWithAuth('/heatmap/themes');
  }

  async getTheme(slug) {
    return this.fetchWithAuth(`/heatmap/themes/${slug}`);
  }

  async createTheme(themeData) {
    return this.fetchWithAuth('/heatmap/themes', {
      method: 'POST',
      body: JSON.stringify(themeData),
    });
  }

  async updateTheme(slug, themeData) {
    return this.fetchWithAuth(`/heatmap/themes/${slug}`, {
      method: 'PUT',
      body: JSON.stringify(themeData),
    });
  }

  async deleteTheme(slug) {
    return this.fetchWithAuth(`/heatmap/themes/${slug}`, {
      method: 'DELETE',
    });
  }

  async setDefaultTheme(slug) {
    return this.fetchWithAuth(`/heatmap/themes/${slug}/set-default`, {
      method: 'POST',
    });
  }

  async duplicateTheme(slug, newName, newSlug) {
    return this.fetchWithAuth(`/heatmap/themes/${slug}/duplicate`, {
      method: 'POST',
      body: JSON.stringify({ new_name: newName, new_slug: newSlug }),
    });
  }

  // Heatmap Generation Settings
  async getGenerationSettings() {
    return this.fetchWithAuth('/heatmap/settings');
  }

  async updateGenerationSettings(settings) {
    return this.fetchWithAuth('/heatmap/settings', {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
  }

  // Heatmap Generation
  async triggerGeneration() {
    return this.fetchWithAuth('/heatmap/generate', {
      method: 'POST',
    });
  }

  async triggerThemeGeneration(themeSlug) {
    return this.fetchWithAuth(`/heatmap/generate/${themeSlug}`, {
      method: 'POST',
    });
  }

  async listGeneratedHeatmaps() {
    return this.fetchWithAuth('/heatmap/generated');
  }

  async listGenerationJobs(status = null, limit = 50) {
    const params = new URLSearchParams();
    if (status) params.append('status', status);
    params.append('limit', limit.toString());

    const queryString = params.toString();
    return this.fetchWithAuth(`/heatmap/jobs?${queryString}`);
  }

  // Get heatmap embed URL
  getHeatmapEmbedUrl(username, themeSlug, format = 'png') {
    return `${API_BASE_URL}/embed/${username}/${themeSlug}.${format}`;
  }

  // Preview theme with parameters
  async previewTheme(themeData) {
    return this.fetchWithAuth('/heatmap/preview', {
      method: 'POST',
      body: JSON.stringify(themeData),
    });
  }

  // Platform Sync
  async triggerSync() {
    return this.fetchWithAuth('/sync/trigger', {
      method: 'POST',
    });
  }

  async getSyncStatus() {
    return this.fetchWithAuth('/sync/status');
  }
}

export default new ApiClient();
