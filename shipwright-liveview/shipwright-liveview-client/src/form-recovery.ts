export class FormRecovery {
  private forms: Map<string, FormData> = new Map()
  private observer: MutationObserver | null = null

  start(element: Element) {
    // Save form state before any changes
    this.saveAllForms(element)

    // Watch for DOM changes
    this.observer = new MutationObserver((mutations) => {
      mutations.forEach(mutation => {
        if (mutation.type === 'childList') {
          mutation.removedNodes.forEach(node => {
            if (node instanceof Element) {
              this.saveForms(node)
            }
          })
          
          mutation.addedNodes.forEach(node => {
            if (node instanceof Element) {
              this.restoreForms(node)
            }
          })
        }
      })
    })

    this.observer.observe(element, {
      childList: true,
      subtree: true
    })
  }

  stop() {
    if (this.observer) {
      this.observer.disconnect()
      this.observer = null
    }
    this.forms.clear()
  }

  private saveAllForms(element: Element) {
    const forms = element.querySelectorAll('form')
    forms.forEach(form => this.saveForm(form))
  }

  private saveForms(element: Element) {
    if (element instanceof HTMLFormElement) {
      this.saveForm(element)
    } else {
      const forms = element.querySelectorAll('form')
      forms.forEach(form => this.saveForm(form))
    }
  }

  private restoreForms(element: Element) {
    if (element instanceof HTMLFormElement) {
      this.restoreForm(element)
    } else {
      const forms = element.querySelectorAll('form')
      forms.forEach(form => this.restoreForm(form))
    }
  }

  private saveForm(form: HTMLFormElement) {
    const formId = this.getFormId(form)
    if (!formId) return

    const formData = new FormData()
    const elements = form.elements

    for (let i = 0; i < elements.length; i++) {
      const element = elements[i] as HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement
      
      if (!element.name) continue

      if (element instanceof HTMLInputElement) {
        if (element.type === 'checkbox' || element.type === 'radio') {
          if (element.checked) {
            formData.append(element.name, element.value)
          }
        } else if (element.type !== 'file' && element.type !== 'submit' && element.type !== 'button') {
          formData.append(element.name, element.value)
        }
      } else if (element instanceof HTMLTextAreaElement || element instanceof HTMLSelectElement) {
        formData.append(element.name, element.value)
      }
    }

    this.forms.set(formId, formData)
  }

  private restoreForm(form: HTMLFormElement) {
    const formId = this.getFormId(form)
    if (!formId) return

    const savedData = this.forms.get(formId)
    if (!savedData) return

    const elements = form.elements

    for (let i = 0; i < elements.length; i++) {
      const element = elements[i] as HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement
      
      if (!element.name) continue

      const savedValues = savedData.getAll(element.name)
      if (savedValues.length === 0) continue

      if (element instanceof HTMLInputElement) {
        if (element.type === 'checkbox' || element.type === 'radio') {
          element.checked = savedValues.includes(element.value)
        } else if (element.type !== 'file') {
          element.value = savedValues[0] as string
        }
      } else if (element instanceof HTMLTextAreaElement || element instanceof HTMLSelectElement) {
        element.value = savedValues[0] as string
      }
    }

    // Restore cursor position for focused elements
    const activeElement = document.activeElement
    if (activeElement && form.contains(activeElement)) {
      if (activeElement instanceof HTMLInputElement || activeElement instanceof HTMLTextAreaElement) {
        const cursorPos = activeElement.selectionStart
        if (cursorPos !== null) {
          activeElement.setSelectionRange(cursorPos, cursorPos)
        }
      }
    }
  }

  private getFormId(form: HTMLFormElement): string | null {
    // Use form's id, name, or generate based on action
    return form.id || form.name || (form.action ? btoa(form.action) : null)
  }
}