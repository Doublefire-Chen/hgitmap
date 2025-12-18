import { useTheme } from '../context/ThemeContext';
import heatmapDarkSvg from '../assets/heatmap-dark.svg';
import heatmapLightSvg from '../assets/heatmap-light.svg';

const HeatmapLogo = ({ size = 48 }) => {
  const { theme } = useTheme();
  const logoSrc = theme === 'dark' ? heatmapDarkSvg : heatmapLightSvg;

  return (
    <img
      src={logoSrc}
      alt="Heatmap Logo"
      width={size}
      height={size}
      style={{ display: 'block' }}
    />
  );
};

export default HeatmapLogo;
