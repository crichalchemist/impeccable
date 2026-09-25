package ui
import "github.com/charmbracelet/lipgloss"
func short(title string) string { return title[:20] + "..." } // flag
func head(b []byte) []byte { return b[:4] } // pass
