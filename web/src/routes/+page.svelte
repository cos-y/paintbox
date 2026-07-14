<script>
  let inputColorRgb = $state();
  let display = $derived(inputColorRgb);
</script>

<input type="color" bind:value={inputColorRgb}/>
<div>{display}</div>
