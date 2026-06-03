import React, { useState, useEffect } from "react";
import styled from "styled-components";

// Intentionally messy formatting
  const Container = styled.div`
    display:flex;
  padding:  20px;
  `;

export function App() {
const [data,setData]=useState(null);

    useEffect(()=>{
        fetch("/api/data").then(res => res.json()).then(d=>setData(d))
    }, []);

// very long line exceeding 100 characters to test line wrapping behavior of the formatter for react components
  return (
    <Container>
      <h1>Dashboard</h1>
      {data ? <div>Data loaded</div> : <div>Loading...</div>}
    </Container>
  );
}
