package ui
import "github.com/charmbracelet/lipgloss"
var title = lipgloss.NewStyle().Foreground(lipgloss.Blend1D(0.5, a, b)) // flag: blend on a title
var bar = lipgloss.NewStyle().Foreground(lipgloss.Blend1D(0.5, a, b))   // pass: not a title
var plain = lipgloss.NewStyle().Foreground(lipgloss.Color("4"))         // pass
