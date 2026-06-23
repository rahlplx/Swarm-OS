import { HardwareDisplay } from "./components/hardware/HardwareDisplay";

function App() {
  return (
    <main style={{ padding: "2rem", fontFamily: "Inter, system-ui, sans-serif" }}>
      <h1>Swarm-OS</h1>
      <p>Your idle GPU earns. Your AI runs free.</p>
      <HardwareDisplay />
    </main>
  );
}

export default App;
