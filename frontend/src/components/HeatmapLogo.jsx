import heatmapLogoSvg from '../assets/heatmap-logo.svg';

const HeatmapLogo = ({ size = 48 }) => {
  return (
    <img
      src={heatmapLogoSvg}
      alt="Heatmap Logo"
      width={size}
      height={size}
      style={{ display: 'block', borderRadius: '50%', overflow: 'hidden' }}
    />
  );
};

export default HeatmapLogo;
