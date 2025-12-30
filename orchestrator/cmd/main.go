package main

import (
	"fmt"
	"log"
	"time"
)

// Node represents a participant or anchor in the microgrid.
type Node struct {
	ID             string
	Type           string // "anchor" or "participant"
	BatteryKWh     float64
	CurrentLoadKW  float64
	IsOnline       bool
}

// MicrogridOrchestrator manages the state of the street.
type MicrogridOrchestrator struct {
	Nodes map[string]*Node
}

func NewOrchestrator() *MicrogridOrchestrator {
	return &MicrogridOrchestrator{
		Nodes: make(map[string]*Node),
	}
}

func (m *MicrogridOrchestrator) RegisterNode(id string, nodeType string) {
	m.Nodes[id] = &Node{
		ID:       id,
		Type:     nodeType,
		IsOnline: true,
	}
	log.Printf("Registered Node: %s (%s)", id, nodeType)
}

func (m *MicrogridOrchestrator) Monitor() {
	// Simple mock loop
	for {
		log.Println("Orchestrator heartbeat...")
		// Logic to query nodes would go here
		time.Sleep(5 * time.Second)
	}
}

func main() {
	fmt.Println("StreetGrid Orchestrator v0.1.0")

	orch := NewOrchestrator()
	orch.RegisterNode("anchor_01", "anchor")
	orch.RegisterNode("participant_01", "participant")

	// Start monitoring (blocking for now)
	orch.Monitor()
}
