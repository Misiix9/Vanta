import re

with open('src/lib/components/SettingsView.svelte', 'r') as f:
    content = f.read()

# Add activeSection state and toggleSection function
state_injection = """
    let activeSection = $state("Theme Profile");

    function toggleSection(name: string) {
        activeSection = activeSection === name ? "" : name;
    }
"""
content = re.sub(r'(let rebuilding = \$state\(false\);)', r'\1\n' + state_injection, content)

# Function to replace sections
def replace_section(match):
    full_match = match.group(0)
    title = match.group(1)
    inner_content = match.group(2)
    
    # We don't want to replace "Colors (Overrides)" h3, only the main ones
    # The regex already captures the first h3 as the main section title.
    return f"""<div class="accordion-item" class:active={{activeSection === "{title}"}}>
            <button class="accordion-header" onclick={{() => toggleSection("{title}")}}>
                <h3>{title}</h3>
                <svg class="accordion-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
            </button>
            {{#if activeSection === "{title}"}}
                <div class="accordion-content">
{inner_content}
                </div>
            {{/if}}
        </div>"""

# Match <section>...</section>
# Note: we need to handle nested tags, but section shouldn't contain other sections.
pattern = re.compile(r'<section>\s*<h3>(.*?)</h3>\s*(.*?)\s*</section>', re.DOTALL)
content = pattern.sub(replace_section, content)

with open('src/lib/components/SettingsView.svelte', 'w') as f:
    f.write(content)
print("done")
