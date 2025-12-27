import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import './OAuthCallback.css';

function OAuthCallback() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const [status, setStatus] = useState('processing');
  const [error, setError] = useState(null);

  useEffect(() => {
    const handleOAuthCallback = async () => {
      // Check for OAuth success parameter
      const oauthStatus = searchParams.get('oauth');

      if (oauthStatus === 'success') {
        setStatus('success');
        setTimeout(() => {
          navigate('/');
        }, 2000);
        return;
      }

      // Handle error from OAuth provider or backend
      const errorParam = searchParams.get('error');
      if (errorParam) {
        setStatus('error');
        // Decode the error message
        const decodedError = decodeURIComponent(errorParam);
        setError(decodedError);
        return;
      }

      // If we get here without success or error, something went wrong
      setStatus('error');
      setError('Invalid OAuth callback. Please try connecting again.');
    };

    handleOAuthCallback();
  }, [searchParams, navigate]);

  return (
    <div className="oauth-callback-page">
      <div className="oauth-callback-container">
        {status === 'processing' && (
          <div className="oauth-status">
            <div className="spinner"></div>
            <h2>Connecting your account...</h2>
            <p>Please wait while we complete the connection.</p>
          </div>
        )}

        {status === 'success' && (
          <div className="oauth-status success">
            <div className="success-icon">✓</div>
            <h2>Successfully connected!</h2>
            <p>Your account has been connected. Redirecting to your dashboard...</p>
          </div>
        )}

        {status === 'error' && (
          <div className="oauth-status error">
            <div className="error-icon">✕</div>
            <h2>Connection failed</h2>
            <p className="error-message">{error}</p>
            <button className="btn btn-primary" onClick={() => navigate('/')}>
              Return to Dashboard
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

export default OAuthCallback;
