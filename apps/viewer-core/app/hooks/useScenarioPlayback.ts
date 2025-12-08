import { useRef, useState } from 'react'
import type { ArchitectureJSON } from '@sruja/viewer'
import { Colors } from '@sruja/shared/utils/cssVars'

export function useScenarioPlayback({ data, viewerRef, followCamera }: { data: ArchitectureJSON | null, viewerRef: React.MutableRefObject<any>, followCamera: boolean }) {
  const playTimerRef = useRef<number | null>(null)
  const [isPlaying, setIsPlaying] = useState(false)
  const [currentScenarioId, setCurrentScenarioId] = useState<string | null>(null)
  const [scenarioStepIndex, setScenarioStepIndex] = useState(0)

  const resolveIdForCy = (id: string) => {
    const cy = viewerRef.current?.cyInstance
    if (!cy) return id
    const direct = cy.getElementById(id)
    if (direct && direct.length > 0) return id
    if (data?.architecture.systems) {
      for (const system of data.architecture.systems) {
        const qualified = `${system.id}.${id}`
        const node = cy.getElementById(qualified)
        if (node && node.length > 0) return qualified
      }
    }
    return id
  }

  const runScenarioStep = (scenario: any, idx: number) => {
    const cy = viewerRef.current?.cyInstance
    if (!cy) return
    const step = scenario.steps[idx]
    if (!step) return
    const fromId = resolveIdForCy(step.from)
    const toId = resolveIdForCy(step.to)
    const fromNode = cy.getElementById(fromId)
    const toNode = cy.getElementById(toId)
    cy.elements().removeClass('scenario-highlight')
    cy.elements().style('background-color', '')
    cy.elements().style('border-color', '')
    cy.elements().style('border-width', '')
    fromNode.addClass('scenario-highlight')
    toNode.addClass('scenario-highlight')
    fromNode.style('background-color', Colors.primary50())
    toNode.style('background-color', Colors.primary50())
    fromNode.style('border-color', Colors.primary())
    toNode.style('border-color', Colors.primary())
    fromNode.style('border-width', '3px')
    toNode.style('border-width', '3px')
    let edge = cy.edges().filter((e: any) => e.data('source') === fromId && e.data('target') === toId)
    if (edge && edge.length > 0) {
      edge.style('line-color', Colors.primary())
      edge.style('width', '3')
      edge.style('target-arrow-color', Colors.primary())
    }
    if (followCamera) {
      cy.animate({ center: { eles: fromNode.union(toNode) } }, { duration: 500 })
    }
  }

  const playScenario = (scenarioId: string) => {
    const scenario = data?.architecture.scenarios?.find(s => s.id === scenarioId)
    if (!scenario || !scenario.steps || scenario.steps.length === 0) return
    setCurrentScenarioId(scenarioId)
    setIsPlaying(true)
    setScenarioStepIndex(0)
    const cy = viewerRef.current?.cyInstance
    if (!cy) return
    const runStep = (idx: number) => runScenarioStep(scenario, idx)
    runStep(0)
    if (playTimerRef.current) window.clearInterval(playTimerRef.current)
    playTimerRef.current = window.setInterval(() => {
      setScenarioStepIndex(prev => {
        const next = prev + 1
        if (!scenario.steps[next]) {
          if (playTimerRef.current) window.clearInterval(playTimerRef.current)
          setIsPlaying(false)
          return prev
        }
        runStep(next)
        return next
      })
    }, 2000)
  }

  const stopScenario = () => {
    if (playTimerRef.current) window.clearInterval(playTimerRef.current)
    setIsPlaying(false)
    setCurrentScenarioId(null)
    setScenarioStepIndex(0)
    const cy = viewerRef.current?.cyInstance
    if (cy) {
      cy.elements().removeClass('scenario-highlight')
      cy.elements().style('background-color', '')
      cy.elements().style('border-color', '')
      cy.elements().style('border-width', '')
      cy.edges().style('line-color', Colors.neutral500())
      cy.edges().style('target-arrow-color', Colors.neutral500())
      cy.edges().style('width', '2')
    }
  }

  const nextScenarioStep = () => {
    if (!currentScenarioId) return
    const scenario = data?.architecture.scenarios?.find(s => s.id === currentScenarioId)
    if (!scenario || !scenario.steps) return
    const next = Math.min(scenarioStepIndex + 1, scenario.steps.length - 1)
    setScenarioStepIndex(next)
    runScenarioStep(scenario, next)
  }

  const prevScenarioStep = () => {
    if (!currentScenarioId) return
    const scenario = data?.architecture.scenarios?.find(s => s.id === currentScenarioId)
    if (!scenario || !scenario.steps) return
    const prev = Math.max(scenarioStepIndex - 1, 0)
    setScenarioStepIndex(prev)
    runScenarioStep(scenario, prev)
  }

  return { isPlaying, currentScenarioId, scenarioStepIndex, playScenario, stopScenario, nextScenarioStep, prevScenarioStep }
}

