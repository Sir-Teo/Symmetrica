// Typing animation for hero
function typeWriter(element, text, speed = 50) {
    let i = 0;
    element.textContent = '';
    
    function type() {
        if (i < text.length) {
            element.textContent += text.charAt(i);
            i++;
            setTimeout(type, speed);
        }
    }
    type();
}

// Initialize syntax highlighting
document.addEventListener('DOMContentLoaded', () => {
    hljs.highlightAll();
    
    // Smooth scroll for anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });
    
    // Navbar scroll effect
    let lastScroll = 0;
    const navbar = document.querySelector('.navbar');
    
    window.addEventListener('scroll', () => {
        const currentScroll = window.pageYOffset;
        
        if (currentScroll <= 0) {
            navbar.style.boxShadow = 'none';
        } else {
            navbar.style.boxShadow = '0 2px 10px rgba(0, 0, 0, 0.3)';
        }
        
        lastScroll = currentScroll;
    });
    
    // Intersection Observer for fade-in animations
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };
    
    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
            }
        });
    }, observerOptions);
    
    // Observe all feature cards and showcase items
    document.querySelectorAll('.feature-card, .showcase-item, .quickstart-step').forEach(el => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(20px)';
        el.style.transition = 'opacity 0.6s ease-out, transform 0.6s ease-out';
        observer.observe(el);
    });
    
    // Add copy button to code blocks
    document.querySelectorAll('pre code').forEach(block => {
        const wrapper = block.closest('pre');
        const copyButton = document.createElement('button');
        copyButton.className = 'copy-button';
        copyButton.textContent = 'Copy';
        copyButton.addEventListener('click', () => {
            navigator.clipboard.writeText(block.textContent).then(() => {
                copyButton.textContent = 'Copied!';
                setTimeout(() => {
                    copyButton.textContent = 'Copy';
                }, 2000);
            });
        });
        wrapper.style.position = 'relative';
        wrapper.appendChild(copyButton);
    });
    
    // Stats counter animation
    const animateCounter = (element, target) => {
        const duration = 2000;
        const start = 0;
        const increment = target / (duration / 16);
        let current = start;
        
        const timer = setInterval(() => {
            current += increment;
            if (current >= target) {
                element.textContent = target;
                clearInterval(timer);
            } else {
                element.textContent = Math.floor(current);
            }
        }, 16);
    };
    
    // Trigger counter animation when stats come into view
    const statsObserver = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const statValue = entry.target.querySelector('.stat-value');
                if (statValue) {
                    const text = statValue.textContent;
                    const number = parseInt(text.replace(/\D/g, ''));
                    if (!isNaN(number)) {
                        animateCounter(statValue, number);
                        statsObserver.unobserve(entry.target);
                    }
                }
            }
        });
    }, { threshold: 0.5 });
    
    document.querySelectorAll('.stat').forEach(stat => {
        statsObserver.observe(stat);
    });
    
    // Scroll reveal animations
    const revealObserver = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('active');
                revealObserver.unobserve(entry.target);
            }
        });
    }, {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    });
    
    // Add reveal class to sections
    document.querySelectorAll('section').forEach((section, index) => {
        if (index > 0) { // Skip first section (hero)
            section.classList.add('reveal');
            revealObserver.observe(section);
        }
    });
    
    // Parallax scrolling effect for hero
    let ticking = false;
    window.addEventListener('scroll', () => {
        if (!ticking) {
            window.requestAnimationFrame(() => {
                const hero = document.querySelector('.hero');
                if (hero) {
                    const scrolled = window.pageYOffset;
                    const heroContent = hero.querySelector('.hero-content');
                    const heroDemo = hero.querySelector('.hero-demo');
                    
                    if (heroContent && scrolled < window.innerHeight) {
                        heroContent.style.transform = `translateY(${scrolled * 0.3}px)`;
                        heroContent.style.opacity = 1 - (scrolled / 500);
                    }
                    if (heroDemo && scrolled < window.innerHeight) {
                        heroDemo.style.transform = `translateY(${scrolled * 0.15}px)`;
                    }
                }
                ticking = false;
            });
            ticking = true;
        }
    });
    
    // Tab switching
    document.querySelectorAll('.tab-button').forEach(button => {
        button.addEventListener('click', () => {
            const tabId = button.getAttribute('data-tab');
            const container = button.closest('.tabs-container');
            
            // Remove active from all buttons in this container
            container.querySelectorAll('.tab-button').forEach(btn => {
                btn.classList.remove('active');
            });
            
            // Remove active from all panes in this container
            container.querySelectorAll('.tab-pane').forEach(pane => {
                pane.classList.remove('active');
            });
            
            // Add active to clicked button
            button.classList.add('active');
            
            // Add active to corresponding pane
            const targetPane = container.querySelector(`#${tabId}-tab`);
            if (targetPane) {
                targetPane.classList.add('active');
                // Re-highlight code in the active pane
                targetPane.querySelectorAll('pre code').forEach(block => {
                    hljs.highlightElement(block);
                });
            }
        });
    });
    
    // Animate benchmark bars when they come into view
    const benchmarkObserver = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.animationPlayState = 'running';
                benchmarkObserver.unobserve(entry.target);
            }
        });
    }, { threshold: 0.5 });
    
    document.querySelectorAll('.benchmark-bar').forEach(bar => {
        bar.style.animationPlayState = 'paused';
        benchmarkObserver.observe(bar);
    });
    
    // Add loading animation
    document.body.classList.add('loaded');
});

// Add copy button styles dynamically
const style = document.createElement('style');
style.textContent = `
    .copy-button {
        position: absolute;
        top: 0.5rem;
        right: 0.5rem;
        padding: 0.25rem 0.75rem;
        background: rgba(99, 102, 241, 0.8);
        color: white;
        border: none;
        border-radius: 0.25rem;
        font-size: 0.75rem;
        cursor: pointer;
        opacity: 0;
        transition: opacity 0.3s, background 0.3s;
    }
    
    pre:hover .copy-button {
        opacity: 1;
    }
    
    .copy-button:hover {
        background: rgba(99, 102, 241, 1);
    }
`;
document.head.appendChild(style);
