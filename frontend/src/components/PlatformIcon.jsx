import { useTheme } from '../context/ThemeContext';
import giteaLogo from '../assets/gitea.svg';
import gitlabLogo from '../assets/gitlab.svg';
import githubLight from '../assets/github-light.svg';
import githubDark from '../assets/github-dark.svg';

const PlatformIcon = ({ platform, size = 20 }) => {
  const { theme } = useTheme();

  const iconStyle = {
    width: size,
    height: size,
    display: 'inline-block',
    verticalAlign: 'middle',
  };

  switch (platform.toLowerCase()) {
    case 'github':
      return (
        <img
          src={theme === 'light' ? githubLight : githubDark}
          alt="GitHub"
          style={iconStyle}
        />
      );
    case 'gitlab':
      return <img src={gitlabLogo} alt="GitLab" style={iconStyle} />;
    case 'gitea':
      return <img src={giteaLogo} alt="Gitea" style={iconStyle} />;
    default:
      return null;
  }
};

export default PlatformIcon;
