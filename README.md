# StreetGridOS
🛡️ Project StreetGrid OS
A Decentralized, Software-Defined Microgrid for Community Resilience

## Status: MVP In Progress
We are building the foundational blocks for the StreetGrid OS.
*   **M1: Digital Twin (Simulation):** Python model verifying load balancing logic.
*   **M1.5: Firmware Skeleton (Rust):** Initial structure for the Edge Node on Raspberry Pi Zero.
*   **Orchestrator Skeleton (Go):** Initial structure for the central coordinator.

### 1. Simulation (The Digital Twin)
The simulation models the energy physics and control logic of a neighborhood.
*   **Location:** `simulation/`
*   **Run:**
    ```bash
    cd simulation
    pip install -r requirements.txt
    python3 src/main.py scenarios/grid_failure.json
    ```

### 2. Firmware (The Edge Node)
The firmware runs on the Raspberry Pi Zero 2W at each house. It handles safety (relays) and telemetry.
*   **Location:** `firmware/`
*   **Language:** Rust
*   **Build & Test:**
    ```bash
    cd firmware
    cargo build
    cargo test
    cargo run
    ```

### 3. Orchestrator (The Coordinator)
The orchestrator manages the high-level state of the microgrid.
*   **Location:** `orchestrator/`
*   **Language:** Go
*   **Run:**
    ```bash
    cd orchestrator
    go run cmd/main.go
    ```

---

1. Vision Statement
To transform a standard residential street into a self-sustaining "energy island" during national grid failures. By leveraging distributed informatics and resilient hardware, StreetGrid OS enables neighbors to pool solar and battery resources to maintain "near-normal" functionality through cooperative load-balancing and frequency-based control.
2. The Core Technical Architecture
The system operates on a three-tier hierarchy designed for maximum energy efficiency, physical safety, and "grid-down" reliability.
A. The Physical Layer (Topology Options)
StreetGrid OS supports two physical deployment topologies, allowing deployment either by ad-hoc civilian cooperatives or by municipal authorities during civil defense scenarios.
Option 1: The "Ad-Hoc" Mesh (Citizen-Led)
 * Scenario: Civilian-led deployment during unplanned grid collapse without utility support.
 * Connection: Neighbors physically connect houses using heavy-gauge private AC cabling (e.g., generator extension cords) or buried private lines.
 * Isolation: Each house manually isolates itself from the street grid via a Transfer Switch (Break-Before-Make) to prevent back-feeding the utility lines.
 * Pros: Legal on private property; no bureaucracy.
 * Cons: Limited power capacity; physically cumbersome setup.
Option 2: The "Transformer Island" (Government/Utility-Led)
 * Scenario: State-sanctioned "Resilience Segments" activated during cyber-attacks or national emergencies.
 * Connection: Utilizes existing utility power lines to share energy among all homes connected to a single Distribution Transformer.
 * Critical Hardware: A Microgrid Interconnection Device (MID) installed at the transformer. This intelligent switchgear automatically severs the connection to the main grid upon failure, creating a safe, floating neutral island for the neighborhood.
 * Safety Mechanism: The MID guarantees galvanic isolation from the medium-voltage utility grid, preventing lethal "Step-Up" back-feed to upstream lines.
B. The Hardware Prerequisites (Universal)
Regardless of the topology chosen above, the network relies on specific "Master/Slave" physics to maintain AC stability:
 * The Anchor Node (Master): At least one house (or a municipal battery skid) must possess a Grid-Forming Inverter capable of "Black Start" and "Frequency Shift Power Control". This unit provides the voltage reference and 60Hz/50Hz heartbeat.
 * The Participant Nodes (Slaves): Standard Grid-Tie Inverters compliant with UL 1741 SA (Rule 21). These inverters must be configured to respond to frequency shifting to automatically throttle generation without digital intervention.
 * The Edge Controller (StreetGrid Node):
   * Compute: Raspberry Pi Zero 2W (low idle power ~0.7W).
   * Radio: SX126x LoRa HAT for long-range, internet-independent telemetry.
   * Sensing: Non-invasive CT Clamps to read real-time current/voltage at the main breaker.
C. The Communication Layer
 * Protocol: Custom binary protocol using Protocol Buffers (Protobuf).
 * Efficiency: No JSON/Text-based overhead. Data is bit-packed to respect LoRa duty cycles (1%) and low bandwidth.
 * Topology: LoRa P2P (Mesh) or LoRaWAN (Star) depending on gateway availability.
 * Function: Handles "Economic Dispatch" (who is allowed to use power) and System Visualization. It does not handle sub-second frequency stabilization (which is done by the Inverters).
D. The Intelligence Layer
 * Language: Rust for the Edge (sensing/safety) to ensure memory safety and zero garbage collection overhead. Go for the Orchestrator/UX (high-level logic and dashboarding).
 * State Machine: A control loop that monitors the microgrid’s "heartbeat" and sheds load (via smart relays) if energy scarcity is detected.
3. MVP Roadmap & Milestones
| Milestone | Phase | Key Deliverable | Status |
|---|---|---|---|
| M1: The Digital Twin | Simulation | Python model proving that 10 houses can balance load/supply without crashing. | ✅ Completed |
| M1.5: The Hardware Interface | Safety | Rust code successfully toggling a relay and reading an ADC on the Pi Zero. | 🚧 In Progress |
| M2: The Shadow Meter | Edge Hardware | RPi Zero 2W successfully reading real-time household wattage via Rust. | |
| M3: The Silent Pulse | Communication | Two nodes exchanging state-of-charge packets via LoRa (P2P) with <1s latency. | |
| M4: The Load Shed | Integration | A "Master" node successfully triggers a relay in a "Neighbor" node based on simulated energy scarcity. | |
4. Key Considerations for Success
⚡ Energy Efficiency
 * Rust vs Go: Use Rust for edge nodes to keep CPU usage <1%, allowing longer operation on backup batteries.
 * Event-Driven: Avoid polling; use hardware interrupts for electrical sensing.
⚠️ Safety & Risk
 * Islanding: Physical isolation from the national grid is mandatory via Automatic Transfer Switches (ATS) or Manual Interlock Kits.
 * Byzantine Fault Tolerance: The software must handle "bad data" from malfunctioning nodes without crashing the grid.
5. Deployment Prerequisites & Liability
StreetGrid OS is a software orchestration layer. It controls data, not physics.
 * Hardware Responsibility: It is the sole responsibility of the deployment site (neighborhood/HOA/Municipality) to procure, install, and certify the high-voltage power electronics (Inverters, Cabling, and Transfer Switches).
 * Electrical Code Compliance: All physical AC interconnections between structures must adhere to local standards.
 * The "Bring Your Own Device" Rule: To join a StreetGrid network, a participant must provide:
   * A compliant Grid-Tie Inverter (UL 1741 SA).
   * A physical connection point to the shared AC bus.
   * A StreetGrid Edge Node (RPi Zero) powered by a small UPS.
