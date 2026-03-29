<script lang="ts">
  interface Props {
    values: number[];
    width?: number;
    height?: number;
    color?: string;
    showArea?: boolean;
  }

  let {
    values,
    width = 60,
    height = 16,
    color = 'var(--accent)',
    showArea = true
  }: Props = $props();

  let latestValue = $derived(values.length > 0 ? values[values.length - 1] : 0);

  let currentColor = $derived(
    latestValue >= 80 ? 'var(--danger)' :
    latestValue >= 50 ? 'var(--warning)' :
    color
  );

  let points = $derived.by(() => {
    if (values.length < 2) return '';
    const maxIndex = values.length - 1;
    return values.map((val, i) => {
      const x = (i / maxIndex) * width;
      const clampedVal = Math.max(0, Math.min(100, val));
      const y = height - (clampedVal / 100) * height;
      return `${x},${y}`;
    }).join(' ');
  });

  let areaPoints = $derived(
    points ? `${points} ${width},${height} 0,${height}` : ''
  );
</script>

{#if values.length >= 2}
  <svg 
    {width} 
    {height} 
    viewBox="0 0 {width} {height}" 
    class="sparkline" 
    aria-hidden="true"
  >
    {#if showArea}
      <polygon 
        points={areaPoints} 
        fill={currentColor} 
        fill-opacity="0.2"
        stroke="none" 
      />
    {/if}
    <polyline 
      {points} 
      fill="none" 
      stroke={currentColor} 
      stroke-width="1.5"
      stroke-linecap="round" 
      stroke-linejoin="round"
    />
  </svg>
{/if}

<style>
  .sparkline {
    display: inline-block;
    vertical-align: middle;
    overflow: visible;
  }
</style>