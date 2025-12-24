import './ColorSchemePicker.css';

const COLOR_SCHEMES = {
  github_green: {
    label: 'GitHub Green',
    colors: ['#ebedf0', '#9be9a8', '#40c463', '#30a14e', '#216e39']
  },
  github_blue: {
    label: 'GitHub Blue',
    colors: ['#ebedf0', '#9be9ff', '#40c4ff', '#2196f3', '#1565c0']
  },
  halloween: {
    label: 'Halloween',
    colors: ['#ebedf0', '#ffee4a', '#ffc501', '#fe9600', '#03001c']
  },
  winter: {
    label: 'Winter',
    colors: ['#ebedf0', '#b6e3f4', '#66c2e0', '#2e8ab8', '#1a5490']
  },
  ocean: {
    label: 'Ocean',
    colors: ['#ebedf0', '#aadaff', '#5fb3d9', '#2a7fad', '#0a4d80']
  },
  sunset: {
    label: 'Sunset',
    colors: ['#ebedf0', '#ffd89b', '#ff9a56', '#ff6b35', '#d94400']
  },
  forest: {
    label: 'Forest',
    colors: ['#ebedf0', '#c8e6c9', '#81c784', '#43a047', '#2e7d32']
  },
  monochrome: {
    label: 'Monochrome',
    colors: ['#ebedf0', '#b0b0b0', '#808080', '#505050', '#202020']
  },
  rainbow: {
    label: 'Rainbow',
    colors: ['#ebedf0', '#ffeb3b', '#4caf50', '#2196f3', '#9c27b0']
  },
  custom: {
    label: 'Custom',
    colors: ['#ebedf0', '#9be9a8', '#40c463', '#30a14e', '#216e39']
  }
};

function ColorSchemePicker({ value, onChange, customColors }) {
  const handleSchemeClick = (schemeKey) => {
    onChange(schemeKey);
  };

  return (
    <div className="color-scheme-picker">
      {Object.entries(COLOR_SCHEMES).map(([key, scheme]) => {
        const isSelected = value === key;
        const displayColors = key === 'custom' && customColors ? customColors : scheme.colors;

        return (
          <div
            key={key}
            className={`scheme-option ${isSelected ? 'selected' : ''}`}
            onClick={() => handleSchemeClick(key)}
          >
            <div className="scheme-preview">
              {displayColors.map((color, i) => (
                <div
                  key={i}
                  className="color-square"
                  style={{ backgroundColor: color }}
                  title={`Level ${i}: ${color}`}
                />
              ))}
            </div>
            <div className="scheme-label">
              {scheme.label}
              {isSelected && <span className="checkmark">✓</span>}
            </div>
          </div>
        );
      })}
    </div>
  );
}

export default ColorSchemePicker;
