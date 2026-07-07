<script>
  // ── CASE 1: Script block — mixed indentation and spacing ─────────────────
  import { onMount } from 'svelte';

  let count=0;
  let name  =  'World';
  let items = ['one','two','three'];

  function increment ( ) {
      count += 1;
  }

  onMount(()=>{
    console.log('mounted');
  });

  // ── CASE 2: Reactive declarations ─────────────────────────────────────────
  $: doubled = count * 2;
  $: {
    console.log(`count is now ${count}`);
  }
</script>

<style>
  /* ── CASE 3: Component styles — formatting ─────────────────────────────── */
  .container {
    display:flex;
      flex-direction: column;
    align-items:center;
  }

  button {
      padding:8px 16px;
    background : #007bff;
    color:white;
      border:none;
    border-radius: 4px;
    cursor:pointer;
  }

  button:hover {
    background: darken(#007bff, 10%);
  }
</style>

<!-- ── CASE 4: Template markup — attribute spacing ─────────────────────────── -->
<div   class="container"   data-testid="main">
  <h1>Hello {name}!</h1>

  <button   on:click={increment}   aria-label="increment counter">
    Count: {count} (doubled: {doubled})
  </button>

  <!-- ── CASE 5: Each block ────────────────────────────────────────────────── -->
  <ul>
    {#each items as item,index}
      <li>{index}: {item}</li>
    {/each}
  </ul>

  <!-- ── CASE 6: Conditional ───────────────────────────────────────────────── -->
  {#if count > 5}
    <p>Count is greater than 5!</p>
  {:else if count > 0}
    <p>Count is positive.</p>
  {:else}
    <p>Count is zero.</p>
  {/if}
</div>
