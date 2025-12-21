const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

class ApiClient {
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
      console.error('❌ [API] Unauthorized - clearing auth and redirecting to login');
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      window.location.href = '/login';
      throw new Error('Unauthorized');
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

  // Contributions
  async getContributions(from = null, to = null) {
    const params = new URLSearchParams();
    if (from) params.append('from', from);
    if (to) params.append('to', to);

    const queryString = params.toString();
    const url = queryString ? `/contributions?${queryString}` : '/contributions';

    return this.fetchWithAuth(url);
  }

  async getContributionStats() {
    return this.fetchWithAuth('/contributions/stats');
  }

  // Activities
  async getActivities(from = null, to = null, limit = 50, offset = 0) {
    const params = new URLSearchParams();
    if (from) params.append('from', from);
    if (to) params.append('to', to);
    params.append('limit', limit.toString());
    params.append('offset', offset.toString());

    const queryString = params.toString();
    const url = `/activities?${queryString}`;

    return this.fetchWithAuth(url);
  }

  async syncActivities(allYears = false, year = null) {
    const params = new URLSearchParams();
    if (allYears) {
      params.append('all_years', 'true');
    } else if (year) {
      params.append('year', year.toString());
    }
    const queryString = params.toString();
    const url = queryString ? `/activities/sync?${queryString}` : '/activities/sync';

    return this.fetchWithAuth(url, {
      method: 'POST',
    });
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
}

export default new ApiClient();
