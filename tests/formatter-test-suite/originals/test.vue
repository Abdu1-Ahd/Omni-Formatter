<template>
  <!-- ── CASE 1: Template — attribute spacing ──────────────────────────────── -->
  <div   class="container"   :class="{ active: isActive }"   v-if="isVisible">
    <h1>{{ title }}</h1>

    <!-- ── CASE 2: v-for loop ──────────────────────────────────────────────── -->
    <ul>
      <li v-for="(item,index) in items" :key="item.id ?? index">
        {{ index }}: {{ item.label }}
      </li>
    </ul>

    <!-- ── CASE 3: Event handling ───────────────────────────────────────────── -->
    <button   @click="handleClick"   @keyup.enter="handleEnter"   :disabled="isLoading">
      {{ isLoading ? 'Loading...' : 'Submit' }}
    </button>
  </div>
</template>

<script>
// ── CASE 4: Options API — mixed indentation ────────────────────────────────
export default {
  name: 'TestComponent',
    data() {
    return {
        title: 'Hello Vue',
      isActive: false,
        isVisible: true,
      isLoading: false,
      items: [
        {id:1,label:'One'},
        { id: 2, label: 'Two' },
      ],
    };
  },
    computed: {
    doubleItems() {
        return this.items.map(i => ({ ...i , label: i.label + i.label }));
    },
  },
    methods: {
    handleClick ( ) {
        this.isLoading = true;
      setTimeout(()=>{ this.isLoading = false; }, 1000);
    },
    handleEnter ( event ) {
        console.log( 'enter pressed' , event );
    },
  },
};
</script>

<style scoped>
/* ── CASE 5: Scoped styles ────────────────────────────────────────────────── */
.container {
  display:flex;
    flex-direction: column;
  align-items:center;
  padding:16px;
}

button {
    background:#007bff;
  color:white;
    border:none;
  padding:8px 16px;
  border-radius:4px;
}
</style>
