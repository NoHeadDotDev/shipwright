import { DomPatch, PatchOp } from './protocol'

export class DomPatcher {
  private root: Element
  // Cache for future optimizations
  // private nodeCache: WeakMap<Element, number[]> = new WeakMap()

  constructor(root: Element) {
    this.root = root
  }

  applyPatches(patches: DomPatch[]) {
    for (const patch of patches) {
      this.applyPatch(patch)
    }
  }

  private applyPatch(patch: DomPatch) {
    const node = this.findNode(patch.path)
    if (!node) return

    switch (patch.op) {
      case PatchOp.Replace:
        this.replace(node, patch.data)
        break
      case PatchOp.Remove:
        this.remove(node)
        break
      case PatchOp.Insert:
        this.insert(node, patch.data.index, patch.data.html)
        break
      case PatchOp.Update:
        this.update(node, patch.data)
        break
      case PatchOp.SetAttr:
        this.setAttribute(node, patch.data.name, patch.data.value)
        break
      case PatchOp.RemoveAttr:
        this.removeAttribute(node, patch.data)
        break
      case PatchOp.AddClass:
        this.addClass(node, patch.data)
        break
      case PatchOp.RemoveClass:
        this.removeClass(node, patch.data)
        break
      case PatchOp.SetProp:
        this.setProperty(node, patch.data.name, patch.data.value)
        break
    }
  }

  private findNode(path: number[]): Element | null {
    let current: Element = this.root
    
    for (const index of path) {
      const children = current.children
      if (index >= children.length) return null
      current = children[index]
    }
    
    return current
  }

  private replace(node: Element, html: string) {
    const temp = document.createElement('div')
    temp.innerHTML = html
    const newNode = temp.firstElementChild
    
    if (newNode && node.parentNode) {
      node.parentNode.replaceChild(newNode, node)
    }
  }

  private remove(node: Element) {
    node.remove()
  }

  private insert(parent: Element, index: number, html: string) {
    const temp = document.createElement('div')
    temp.innerHTML = html
    const newNode = temp.firstElementChild
    
    if (!newNode) return
    
    if (index >= parent.children.length) {
      parent.appendChild(newNode)
    } else {
      parent.insertBefore(newNode, parent.children[index])
    }
  }

  private update(node: Element, content: string) {
    if (node instanceof HTMLInputElement || 
        node instanceof HTMLTextAreaElement || 
        node instanceof HTMLSelectElement) {
      // Preserve form state
      const activeElement = document.activeElement
      const selectionStart = (node as any).selectionStart
      const selectionEnd = (node as any).selectionEnd
      
      node.value = content
      
      if (activeElement === node && selectionStart !== undefined) {
        (node as any).setSelectionRange(selectionStart, selectionEnd)
      }
    } else {
      node.textContent = content
    }
  }

  private setAttribute(node: Element, name: string, value: string) {
    node.setAttribute(name, value)
  }

  private removeAttribute(node: Element, name: string) {
    node.removeAttribute(name)
  }

  private addClass(node: Element, className: string) {
    node.classList.add(className)
  }

  private removeClass(node: Element, className: string) {
    node.classList.remove(className)
  }

  private setProperty(node: Element, name: string, value: any) {
    (node as any)[name] = value
  }
}

// Efficient DOM differ
export class DomDiffer {
  static diff(oldHtml: string, newHtml: string): DomPatch[] {
    const patches: DomPatch[] = []
    
    const oldDoc = new DOMParser().parseFromString(oldHtml, 'text/html')
    const newDoc = new DOMParser().parseFromString(newHtml, 'text/html')
    
    this.diffNodes(oldDoc.body, newDoc.body, [], patches)
    
    return patches
  }

  private static diffNodes(
    oldNode: Element, 
    newNode: Element, 
    path: number[], 
    patches: DomPatch[]
  ) {
    // Compare attributes
    const oldAttrs = oldNode.attributes
    const newAttrs = newNode.attributes
    
    // Check for removed/changed attributes
    for (let i = 0; i < oldAttrs.length; i++) {
      const attr = oldAttrs[i]
      const newValue = newNode.getAttribute(attr.name)
      
      if (newValue === null) {
        patches.push({
          op: PatchOp.RemoveAttr,
          path,
          data: attr.name
        })
      } else if (newValue !== attr.value) {
        patches.push({
          op: PatchOp.SetAttr,
          path,
          data: { name: attr.name, value: newValue }
        })
      }
    }
    
    // Check for added attributes
    for (let i = 0; i < newAttrs.length; i++) {
      const attr = newAttrs[i]
      if (!oldNode.hasAttribute(attr.name)) {
        patches.push({
          op: PatchOp.SetAttr,
          path,
          data: { name: attr.name, value: attr.value }
        })
      }
    }
    
    // Compare children
    const oldChildren = Array.from(oldNode.children)
    const newChildren = Array.from(newNode.children)
    
    const maxLength = Math.max(oldChildren.length, newChildren.length)
    
    for (let i = 0; i < maxLength; i++) {
      const oldChild = oldChildren[i]
      const newChild = newChildren[i]
      const childPath = [...path, i]
      
      if (!oldChild && newChild) {
        // Insert new child
        patches.push({
          op: PatchOp.Insert,
          path,
          data: { index: i, html: newChild.outerHTML }
        })
      } else if (oldChild && !newChild) {
        // Remove old child
        patches.push({
          op: PatchOp.Remove,
          path: childPath,
          data: null
        })
      } else if (oldChild && newChild) {
        if (oldChild.tagName !== newChild.tagName) {
          // Replace if tag names differ
          patches.push({
            op: PatchOp.Replace,
            path: childPath,
            data: newChild.outerHTML
          })
        } else {
          // Recursively diff children
          this.diffNodes(oldChild, newChild, childPath, patches)
        }
      }
    }
    
    // Compare text content for leaf nodes
    if (oldNode.children.length === 0 && newNode.children.length === 0) {
      const oldText = oldNode.textContent || ''
      const newText = newNode.textContent || ''
      
      if (oldText !== newText) {
        patches.push({
          op: PatchOp.Update,
          path,
          data: newText
        })
      }
    }
  }
}