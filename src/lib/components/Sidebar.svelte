<script lang="ts">
  interface Props {
    currentPath: string;
  }
  let { currentPath }: Props = $props();

  const nav = [
    { href: '/',           label: 'overview',  glyph: '◎' },
    { href: '/portfolio',  label: 'portfolio', glyph: '▤' },
    { href: '/trade',      label: 'trade',     glyph: '↹' },
    { href: '/orders',     label: 'orders',    glyph: '⚑' },
    { href: '/settings',   label: 'settings',  glyph: '⚙' },
  ];

  function isActive(href: string): boolean {
    if (href === '/') return currentPath === '/';
    return currentPath.startsWith(href);
  }
</script>

<aside class="sidebar">
  <nav>
    {#each nav as item}
      <a class="nav-item" class:active={isActive(item.href)} href={item.href}>
        <span class="glyph">{item.glyph}</span>
        <span class="label">{item.label}</span>
      </a>
    {/each}
  </nav>

  <div class="sidebar-foot">
    <div class="kbd-hint">
      <span>⌘K</span>
      <span>commands</span>
    </div>
  </div>
</aside>

<style>
  .sidebar {
    background: var(--bg-1);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }
  nav {
    padding: var(--s-2) 0;
  }
  .nav-item {
    display: grid;
    grid-template-columns: 24px 1fr;
    align-items: center;
    gap: var(--s-2);
    padding: 8px var(--s-4);
    color: var(--fg-1);
    text-decoration: none;
    font-size: var(--fs-base);
    text-transform: lowercase;
    letter-spacing: 0.04em;
    border-left: 2px solid transparent;
    transition: background 60ms ease, color 60ms ease, border-color 60ms ease;
  }
  .nav-item:hover {
    background: var(--bg-2);
    color: var(--fg-0);
  }
  .nav-item.active {
    background: var(--bg-2);
    color: var(--accent);
    border-left-color: var(--accent);
  }
  .glyph {
    font-size: var(--fs-md);
    color: inherit;
    opacity: 0.8;
  }
  .sidebar-foot {
    padding: var(--s-3) var(--s-4);
    border-top: 1px solid var(--border);
  }
  .kbd-hint {
    display: flex;
    gap: var(--s-2);
    align-items: center;
    font-size: var(--fs-xs);
    color: var(--fg-2);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .kbd-hint span:first-child {
    border: 1px solid var(--border-hi);
    padding: 1px 5px;
    font-family: var(--font-data);
    color: var(--fg-1);
  }
</style>
