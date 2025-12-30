import numpy as np
import math

class Node:
    def __init__(self, node_data):
        self.id = node_data['id']
        self.type = node_data['type']
        self.battery_capacity_kwh = node_data.get('battery_capacity_kwh', 0)
        self.solar_capacity_kw = node_data.get('solar_capacity_kw', 0)
        self.base_load_kw = node_data.get('base_load_kw', 0)

        self.current_battery_kwh = self.battery_capacity_kwh * 0.5 # Start at 50%
        self.current_load_kw = 0
        self.current_solar_kw = 0
        self.is_shed = False

    def step(self, hour_of_day):
        """Calculates state for the current hour."""
        self.is_shed = False

        # Simple solar model: Peak at 12:00, zero at night.
        if 6 <= hour_of_day <= 18:
            peak = self.solar_capacity_kw
            self.current_solar_kw = peak * math.exp(-((hour_of_day - 12)**2) / 10)
        else:
            self.current_solar_kw = 0

        # Simple load model
        variation = np.random.normal(0, 0.1)
        evening_bump = 0.5 if 17 <= hour_of_day <= 21 else 0
        self.current_load_kw = max(0, self.base_load_kw + evening_bump + variation)

        return self.current_solar_kw, self.current_load_kw

    def update_battery(self, net_energy_kwh):
        """
        Updates battery state based on net energy (Production - Consumption).
        Returns the remainder (unhandled energy).
        """
        if self.battery_capacity_kwh == 0:
            return net_energy_kwh

        if net_energy_kwh > 0: # Surplus, charge battery
            space = self.battery_capacity_kwh - self.current_battery_kwh
            charge = min(space, net_energy_kwh)
            self.current_battery_kwh += charge
            return net_energy_kwh - charge # Remaining surplus

        else: # Deficit, discharge battery
            needed = -net_energy_kwh
            available = self.current_battery_kwh
            discharge = min(available, needed)
            self.current_battery_kwh -= discharge
            return -(needed - discharge) # Remaining deficit (negative)

class Microgrid:
    def __init__(self, nodes_data):
        self.nodes = [Node(n) for n in nodes_data]
        self.log = []

    def run_step(self, hour_of_day, step_duration_hours=1.0):
        # 1. Calculate Generation and Initial Load
        total_solar = 0
        total_load = 0
        total_battery_capacity = sum(n.battery_capacity_kwh for n in self.nodes)
        total_battery_current = sum(n.current_battery_kwh for n in self.nodes)

        soc = total_battery_current / total_battery_capacity if total_battery_capacity > 0 else 0

        # 2. Pre-emptive Shedding Strategy (The "Intelligence")
        # If SOC is low, shed participants to save the grid (Anchors).
        shed_threshold = 0.35 # Shed if below 35%
        force_shed = soc < shed_threshold

        nodes_shed_count = 0

        for node in self.nodes:
            s, l = node.step(hour_of_day)

            # Apply shedding logic
            if force_shed and node.type == 'participant':
                node.is_shed = True
                l = 0
                nodes_shed_count += 1

            total_solar += s
            total_load += l

        # 3. Calculate Net
        net_power_kw = total_solar - total_load
        net_energy_kwh = net_power_kw * step_duration_hours

        # 4. Battery Balancing
        # If we still have a deficit after pre-emptive shedding, we must shed more?
        # Or just see if battery can cover it.

        # Simulation of battery drain
        current_deficit = -net_energy_kwh if net_energy_kwh < 0 else 0
        total_battery_available = sum(n.current_battery_kwh for n in self.nodes)

        if current_deficit > total_battery_available:
            # Emergency Shedding (Even Anchors might struggle, or remaining participants)
            # This shouldn't happen if we pre-emptively shed, unless Anchors alone overload.
            pass

        remaining_energy = net_energy_kwh
        for node in self.nodes:
            remaining_energy = node.update_battery(remaining_energy)

        grid_status = "Stable"
        if remaining_energy < -0.01:
            grid_status = "Collapsing (Deficit)"
        elif nodes_shed_count > 0:
            grid_status = f"Stable (Shed {nodes_shed_count} nodes)"
        elif remaining_energy > 0.01:
            grid_status = "Curtailed (Surplus)"

        step_log = {
            "hour": hour_of_day,
            "soc": soc,
            "total_solar_kw": total_solar,
            "total_load_kw": total_load,
            "net_grid_kwh": remaining_energy,
            "status": grid_status,
            "total_battery_stored": sum(n.current_battery_kwh for n in self.nodes),
            "nodes_shed": nodes_shed_count
        }
        self.log.append(step_log)
        return step_log
