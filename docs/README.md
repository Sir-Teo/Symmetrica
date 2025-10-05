# Symmetrica GitHub Pages

This directory contains the GitHub Pages site for Symmetrica.

## 🌐 Live Site

Visit: [https://sir-teo.github.io/Symmetrica](https://sir-teo.github.io/Symmetrica)

## 📁 Structure

```
docs/
├── index.html              # Landing page
├── getting-started.html    # Installation and quick start guide
├── examples.html           # Comprehensive examples
├── playground.html         # Interactive playground
├── css/
│   ├── style.css          # Main styles
│   ├── docs.css           # Documentation page styles
│   └── playground.css     # Playground styles
├── js/
│   ├── main.js            # Main JavaScript
│   └── playground.js      # Playground functionality
└── _config.yml            # GitHub Pages configuration
```

## 🚀 Features

### Landing Page
- Modern hero section with gradient text
- Feature showcase grid
- Live code examples
- Architecture diagram
- Quick start guide
- Responsive design

### Documentation
- Modular architecture overview
- Getting started guide
- Comprehensive examples
- API references
- Sidebar navigation

### Interactive Playground
- Pre-loaded examples
- Syntax highlighting
- Code snippets for:
  - Basic operations
  - Differentiation
  - Integration
  - Simplification
  - Equation solving
  - Series expansion

## 🎨 Design

- **Dark theme** optimized for code readability
- **Gradient accents** (purple/blue)
- **Syntax highlighting** with highlight.js
- **Smooth animations** and transitions
- **Fully responsive** mobile-first design

## 🛠️ Technologies

- Pure HTML/CSS/JavaScript (no build step required)
- [highlight.js](https://highlightjs.org/) for syntax highlighting
- Custom CSS with CSS Grid and Flexbox
- Vanilla JavaScript for interactivity

## 📝 Updating Content

### Adding a New Example

Edit `js/playground.js` and add to the `examples` object:

```javascript
newExample: {
    code: `// Your Rust code`,
    output: `Expected output`,
    explanation: `<p>Description</p>`
}
```

### Styling Changes

- **Global styles**: `css/style.css`
- **Documentation pages**: `css/docs.css`
- **Playground**: `css/playground.css`

### Adding New Pages

1. Create `new-page.html`
2. Add navigation link to navbar in all pages
3. Follow existing page structure

## 🚀 Deployment

GitHub Pages automatically builds and deploys when changes are pushed to the `main` branch.

To enable GitHub Pages:
1. Go to repository Settings
2. Navigate to Pages section
3. Source: Deploy from branch `main`
4. Folder: `/docs`
5. Save

Changes are live within 1-2 minutes of pushing.

## 🎯 SEO & Meta

All pages include:
- Meta description
- Proper title tags
- Open Graph tags (can be added)
- Semantic HTML5 structure

## 📱 Responsive Breakpoints

- Desktop: 1200px+
- Tablet: 768px - 1199px
- Mobile: < 768px

## 🔧 Local Development

No build step required! Just open any HTML file in a browser:

```bash
cd docs
python3 -m http.server 8000
# Visit http://localhost:8000
```

Or use any static file server.

## 📄 License

Same as the main project: Dual MIT/Apache-2.0
