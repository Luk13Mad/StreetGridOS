import unittest
import numpy as np
from model import Node, Microgrid

class TestSimulation(unittest.TestCase):
    def setUp(self):
        # Seed random number generator for reproducible tests
        np.random.seed(42)

    def test_node_initialization(self):
        node_data = {
            "id": "test_node",
            "type": "participant",
            "battery_capacity_kwh": 10.0,
            "solar_capacity_kw": 2.0,
            "base_load_kw": 1.0
        }
        node = Node(node_data)
        self.assertEqual(node.current_battery_kwh, 5.0) # Starts at 50%
        self.assertEqual(node.type, "participant")

    def test_solar_generation(self):
        node_data = {"id": "n1", "type": "participant", "solar_capacity_kw": 5.0}
        node = Node(node_data)

        # Noon
        s, l = node.step(12)
        self.assertAlmostEqual(s, 5.0, delta=0.1)

        # Midnight
        s, l = node.step(0)
        self.assertEqual(s, 0)

    def test_battery_charge(self):
        node_data = {"id": "n1", "type": "participant", "battery_capacity_kwh": 10.0}
        node = Node(node_data)
        node.current_battery_kwh = 5.0

        # Charge with 2kWh
        rem = node.update_battery(2.0)
        self.assertEqual(node.current_battery_kwh, 7.0)
        self.assertEqual(rem, 0)

        # Charge with 6kWh (should fill to 10, return 1)
        rem = node.update_battery(6.0) # 7+6 = 13 > 10
        self.assertEqual(node.current_battery_kwh, 10.0)
        self.assertEqual(rem, 3.0)

    def test_grid_shedding(self):
        # Create a grid with low battery that MUST shed
        nodes = [
            {"id": "anchor", "type": "anchor", "battery_capacity_kwh": 1.0, "base_load_kw": 0.1},
            {"id": "p1", "type": "participant", "battery_capacity_kwh": 0, "base_load_kw": 10.0} # Huge load
        ]
        grid = Microgrid(nodes)

        # Scenario:
        # Total Battery Capacity = 1.0
        # If Battery is 0.3, SOC = 30% < 35% Threshold.
        # It SHOULD trigger pre-emptive shedding.
        grid.nodes[0].current_battery_kwh = 0.3

        # Run step at night (no solar)
        log = grid.run_step(0, 1.0)

        # Should have shed p1
        self.assertGreater(log['nodes_shed'], 0)
        self.assertEqual(grid.nodes[1].is_shed, True)
        self.assertEqual(log['status'], "Stable (Shed 1 nodes)")

if __name__ == '__main__':
    unittest.main()
