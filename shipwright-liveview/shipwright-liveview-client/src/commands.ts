import { ClientCommand, CommandType } from './protocol'

export class CommandExecutor {
  execute(commands: ClientCommand[]) {
    for (const command of commands) {
      this.executeCommand(command)
    }
  }

  private executeCommand(command: ClientCommand) {
    const elements = document.querySelectorAll(command.target)
    
    elements.forEach(element => {
      switch (command.type) {
        case CommandType.Show:
          this.show(element as HTMLElement, command.transition)
          break
        case CommandType.Hide:
          this.hide(element as HTMLElement, command.transition)
          break
        case CommandType.Toggle:
          this.toggle(element as HTMLElement, command.transition)
          break
        case CommandType.AddClass:
          this.addClass(element, command.args)
          break
        case CommandType.RemoveClass:
          this.removeClass(element, command.args)
          break
        case CommandType.SetAttribute:
          this.setAttribute(element, command.args.name, command.args.value)
          break
        case CommandType.RemoveAttribute:
          this.removeAttribute(element, command.args)
          break
        case CommandType.Dispatch:
          this.dispatch(element, command.args)
          break
        case CommandType.Push:
          this.push(command.args)
          break
        case CommandType.Focus:
          this.focus(element as HTMLElement)
          break
        case CommandType.Blur:
          this.blur(element as HTMLElement)
          break
      }
    })
  }

  private show(element: HTMLElement, transition?: any) {
    if (transition) {
      this.applyTransition(element, transition, () => {
        element.style.display = ''
      })
    } else {
      element.style.display = ''
    }
  }

  private hide(element: HTMLElement, transition?: any) {
    if (transition) {
      this.applyTransition(element, transition, () => {
        element.style.display = 'none'
      })
    } else {
      element.style.display = 'none'
    }
  }

  private toggle(element: HTMLElement, transition?: any) {
    const isHidden = element.style.display === 'none' || 
                     window.getComputedStyle(element).display === 'none'
    
    if (isHidden) {
      this.show(element, transition)
    } else {
      this.hide(element, transition)
    }
  }

  private addClass(element: Element, className: string) {
    element.classList.add(...className.split(' '))
  }

  private removeClass(element: Element, className: string) {
    element.classList.remove(...className.split(' '))
  }

  private setAttribute(element: Element, name: string, value: string) {
    element.setAttribute(name, value)
  }

  private removeAttribute(element: Element, name: string) {
    element.removeAttribute(name)
  }

  private dispatch(element: Element, eventName: string) {
    element.dispatchEvent(new CustomEvent(eventName, { bubbles: true }))
  }

  private push(eventData: any) {
    // This would send a custom event back to the server
    window.dispatchEvent(new CustomEvent('liveview:push', { detail: eventData }))
  }

  private focus(element: HTMLElement) {
    element.focus()
  }

  private blur(element: HTMLElement) {
    element.blur()
  }

  private applyTransition(
    element: HTMLElement, 
    transition: any, 
    callback: () => void
  ) {
    const duration = transition.duration || 300
    const from = transition.from || {}
    const to = transition.to || {}

    // Apply from styles
    Object.assign(element.style, from)

    // Force reflow
    element.offsetHeight

    // Enable transition
    element.style.transition = `all ${duration}ms ease-in-out`

    // Apply to styles
    requestAnimationFrame(() => {
      Object.assign(element.style, to)
    })

    // Cleanup after transition
    setTimeout(() => {
      element.style.transition = ''
      callback()
    }, duration)
  }
}